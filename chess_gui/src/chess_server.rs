use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chess_engine::game_data::{Gameboard, Piece, WinStatus};
use chess_engine::logic::get_game_state;
use chess_engine::parser;

use crate::{move_piece_with_state, MainState};

const IP_PORT: u16 = 1337;

pub(crate) struct Server {
    pub(crate) ip: String,
    listener: TcpListener,
    pub(crate) clients: Vec<TcpStream>,
    pub(crate) move_client: Option<SocketAddr>,
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
}

impl Server {
    pub(crate) fn spectator_count(&self) -> usize {
        return self.clients.len() - if self.move_client.is_some() { 1 } else { 0 };
    }

    /**Simply calls shutdown on all connected clients, returns true if all shutdowns are successful*/
    pub(crate) fn shutdown(&self) -> bool {
        let mut is_successful = true;
        for client in &self.clients {
            if client.shutdown(std::net::Shutdown::Both).is_err() {
                is_successful = false;
            }
        }

        is_successful
    }
}

pub(crate) fn start_server() -> Option<Server> {
    if let Ok(my_local_ip) = local_ip_address::local_ip() {
        let mut full_ip = my_local_ip.to_string();
        full_ip.push(':');
        full_ip.push_str(&IP_PORT.to_string());
        if let Ok(listener) = TcpListener::bind(&full_ip) {
            listener
                .set_nonblocking(true)
                .expect("failed to initialize non-blocking");

            let clients = vec![];
            let (tx, rx) = mpsc::channel::<(String, SocketAddr)>();

            return Some(Server {
                ip: full_ip,
                listener,
                clients,
                tx,
                rx,
                move_client: None,
            });
        } else {
            println!("Error binding listener for ip {}", full_ip);
        }
    } else {
        println!("Error getting IP");
    }
    None
}

fn get_state_msg(gameboard: &Gameboard, win_status: WinStatus) -> Option<String> {
    let fen = &parser::get_fen(&gameboard.game);
    if fen.is_none() {
        return None;
    }
    let mut win_status_prefix = match win_status {
        WinStatus::WhiteWon => "end:w",
        WinStatus::BlackWon => "end:b",
        WinStatus::Tie => "end:-",
        WinStatus::Nothing => "board:",
    }
    .to_string();
    win_status_prefix.push_str(&fen.as_ref().unwrap());

    return Some(win_status_prefix);
}

fn get_status_msg(main_state: &mut MainState) -> Option<String> {
    let win_status = get_game_state(
        &main_state.active_game.game,
        &main_state.active_game.active_threats,
        true,
    );

    get_state_msg(&main_state.active_game.game, win_status)
}

fn connect_clients(main_state: &mut MainState) {
    if let Some(server) = &mut main_state.server {
        while let Ok((mut socket, addr)) = server.listener.accept() {
            println!("Client {} connected", addr);

            let tx = server.tx.clone();
            server
                .clients
                .push(socket.try_clone().expect("failed to clone client"));

            // as this is not acording to the docs it is removed
            // when a client joins, send them the board
            // tx.send(("get:board".to_string(), addr))
            //    .expect("failed to send msg to rx");

            // spawns a client looper that will keep going forever untill ETO or disconnect message
            thread::spawn(move || {
                let read_buffer = BufReader::new(&mut socket);
                let mut split = read_buffer.split(b';');
                while let Some(read) = split.next() {
                    match read {
                        Ok(msg) => {
                            let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                            println!("{}: {:?}", addr, msg);
                            if msg.starts_with("disconnect") {
                                tx.send((msg, addr)).expect("failed to send msg to rx");
                                return;
                            }
                            tx.send((msg, addr)).expect("failed to send msg to rx");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("closing connection with: {}", addr);
                            break;
                        }
                    }
                }
                tx.send(("disconnect:closed_connection".to_string(), addr))
                    .expect("failed to send msg to rx");
            });
        }
    }
}

fn handle_message(main_state: &mut MainState, msg: String, addr: SocketAddr) {
    let split: Vec<String> = msg.split(":").map(|s| s.to_string()).collect();
    if split.len() != 2 {
        return;
    }

    let action = &split[0];
    let input = &split[1];
    
    const INVALID_STATE_MSG: &str = "err:invalid_state";

    let mut send_msg = msg.clone();
    let mut send_to_all = true;

    match &action[..] {
        // disconnect is an internal command used for logic and not specified in the docs
        // disconnect:[REASON];
        "disconnect" => {
            send_to_all = false;
            println!("Client {} disconnected due to {}", addr, input);
            if let Some(server) = &mut main_state.server {
                for index in 0..server.clients.len() {
                    let client_addr = server.clients[index].peer_addr();
                    if client_addr.is_ok() && client_addr.unwrap() == addr {
                        server.clients.remove(index);
                        break;
                    }
                }
                if Some(addr) == server.move_client {
                    server.move_client = None;
                }
            }
        }
        "init" => {
            let request_player = match input.chars().nth(0) {
                Some('s') => false,
                Some('p') => true,
                None => true,
                _ => false,
            };

            send_to_all = false;
            if let Some(state) = get_status_msg(main_state) {
                send_msg = state;
            } else {
                send_msg = INVALID_STATE_MSG.to_string()
            }

            let mut is_player = false;
            if request_player {
                if let Some(server) = &mut main_state.server {
                    // only one client can send inputs
                    if server.move_client.is_none() {
                        server.move_client = Some(addr);
                        println!("Client {} is now player", addr);
                        is_player = true;
                    }
                }
            }

            if is_player {
                send_msg.push_str(";playertype:p")
            } else {
                send_msg.push_str(";playertype:s")
            }
        }
        "get" => match &input[..] {
            "board" => {
                send_to_all = false;
                if let Some(state) = get_status_msg(main_state) {
                    send_msg = state;
                } else {
                    send_msg = INVALID_STATE_MSG.to_string()
                }
            }
            "spectatorcount" => {
                send_to_all = false;
                let mut result = "spectatorcount:".to_string();
                if let Some(server) = &mut main_state.server {
                    result.push_str(&(server.spectator_count()).to_string());
                }
                send_msg = result;
            }
            _ => return,
        },
        "move" => {
            // if the wrong client sends a move then continue
            if let Some(server) = &mut main_state.server {
                let is_invalid_client = Some(addr) != server.move_client;
                let is_invalid_length = input.len() != 5;
                if is_invalid_client || is_invalid_length {
                    // is wrong client to move or invalid input
                    send_to_all = false;

                    if let Some(state) = get_status_msg(main_state) {
                        let msg = if is_invalid_client {
                            "err:invalid_client"
                        } else {
                            "err:invalid_length"
                        };

                        send_msg = state + ";" + msg;
                    } else {
                        send_msg = INVALID_STATE_MSG.to_string()
                    }
                } else {
                    let (move_from, move_to) = match parser::parse_move(&input[0..4].to_string()) {
                        Some(t) => t,
                        None => return,
                    };

                    let promotion = match input.chars().nth(4) {
                        Some('q') => Piece::Queen,
                        Some('r') => Piece::Rook,
                        Some('b') => Piece::Bishop,
                        Some('n') => Piece::Knight,
                        None => Piece::Queen,
                        _ => Piece::Queen,
                    };

                    let piece_data =
                        main_state.active_game.game.game.board[move_from.x][move_from.y];

                    //validates move and retuns state
                    if !piece_data.is_white && piece_data.piece != Piece::None {
                        let win_status =
                            move_piece_with_state(main_state, move_from, move_to, promotion);
                        main_state.active_game.win_status = win_status;

                        if let Some(state) = get_state_msg(&main_state.active_game.game, win_status)
                        {
                            send_msg = state;
                        } else {
                            send_msg = INVALID_STATE_MSG.to_string()
                        }
                    } else {
                        send_to_all = false;
                        if let Some(state) = get_status_msg(main_state) {
                            send_msg = state + ";err:invalid_mov";
                        } else {
                            send_msg = INVALID_STATE_MSG.to_string()
                        }
                    }
                }
            }
        }
        _ => return,
    }

    send_msg.push(';');
    let buff = send_msg.into_bytes();

    if let Some(server) = &mut main_state.server {
        if send_to_all {
            for client in &mut server.clients {
                let _write_error = client.write_all(&buff);
            }
        } else {
            for client in &mut server.clients {
                let client_addr = client.peer_addr();
                if client_addr.is_ok() && client_addr.unwrap() == addr {
                    let _write_error = client.write_all(&buff);
                }
            }
        }
    }
    return;
}

fn send_pending_move(main_state: &mut MainState) {
    if let Some(server) = &mut main_state.server {
        if main_state.active_game.penging_send {
            main_state.active_game.penging_send = false;
            let win_status = get_game_state(
                &main_state.active_game.game,
                &main_state.active_game.active_threats,
                true,
            );
            if let Some(mut send_msg) = get_state_msg(&main_state.active_game.game, win_status) {
                send_msg.push(';');
                let buff = send_msg.into_bytes();
                for client in &mut server.clients {
                    let _write_error = client.write_all(&buff);
                }
            }
        }
    }
}

/** Will add clients to list and preform all actions recived from all clients, must be called every server tick */
pub(crate) fn server_loop(main_state: &mut MainState) -> Result<(), String> {
    connect_clients(main_state);

    loop {
        if let Some(server) = &main_state.server {
            if let Ok((msg, addr)) = server.rx.try_recv() {
                handle_message(main_state, msg, addr);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    send_pending_move(main_state);

    Ok(())
}
