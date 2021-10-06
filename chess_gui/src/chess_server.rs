use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chess_engine::game_data::{Gameboard, Piece, WinStatus};
use chess_engine::logic::get_game_state;
use chess_engine::parser;

use crate::{move_piece_with_state, MainState};

const LOCAL: &str = "127.0.0.1:1337";

pub(crate) struct Server {
    listener: TcpListener,
    clients: Vec<TcpStream>,
    move_client: Option<SocketAddr>,
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
}

pub(crate) fn start_server() -> Server {
    let listener = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    listener
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");

    let clients = vec![];
    let (tx, rx) = mpsc::channel::<(String, SocketAddr)>();

    return Server {
        listener,
        clients,
        tx,
        rx,
        move_client: None,
    };
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

pub(crate) fn server_loop(main_state: &mut MainState) -> Result<(), String> {
    if let Some(server) = &mut main_state.server {
        while let Ok((mut socket, addr)) = server.listener.accept() {
            println!("Client {} connected", addr);

            let tx = server.tx.clone();
            server
                .clients
                .push(socket.try_clone().expect("failed to clone client"));

            // when a client joins, send them the board
            // tx.send(("get:board".to_string(), addr))
            //    .expect("failed to send msg to rx");

            thread::spawn(move || {
                let read_buffer = BufReader::new(&mut socket);
                let mut split = read_buffer.split(b';');
                while let Some(read) = split.next() {
                    match read {
                        Ok(msg) => {
                            let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                            println!("{}: {:?}", addr, msg);
                            tx.send((msg, addr)).expect("failed to send msg to rx");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            tx.send(("disconnect:closed_connection".to_string(), addr))
                                .expect("failed to send msg to rx");
                            println!("closing connection with: {}", addr);
                            break;
                        }
                    }
                }
            });
        }
    }

    const INVALID_STATE_MSG: &str = "err:invalid_state";

    loop {
        let mut recv: Result<(String, SocketAddr), mpsc::TryRecvError> =
            Err(mpsc::TryRecvError::Empty);

        // this has to be soooo messy because of rusts only one owner
        if let Some(server) = &mut main_state.server {
            recv = server.rx.try_recv();
        }

        // do chess logic here
        if let Ok((msg, addr)) = recv {
            let mut send_msg = msg.clone();
            let mut send_to_all = true;

            let split: Vec<String> = msg.split(":").map(|s| s.to_string()).collect();
            if split.len() != 2 {
                continue;
            }

            let action = &split[0];
            let input = &split[1];
            println!("[{}],[{}]", action, input);
            match &action[..] {
                "disconnect" => {
                    // disconnect:[REASON];
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
                "request" => match &input[..] {
                    "rematch" => {
                        //rematch? not agreed upon yet
                    }
                    _ => (),
                },
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
                            result.push_str(&(server.clients.len() - 1).to_string());
                        }
                        send_msg = result;
                    }
                    _ => continue,
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
                            let (move_from, move_to) =
                                match parser::parse_move(&input[0..4].to_string()) {
                                    Some(t) => t,
                                    None => continue,
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

                            if !piece_data.is_white && piece_data.piece != Piece::None {
                                let win_status = move_piece_with_state(
                                    main_state, move_from, move_to, promotion,
                                );
                                main_state.active_game.win_status = win_status;

                                if let Some(state) =
                                    get_state_msg(&main_state.active_game.game, win_status)
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
                _ => continue,
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
                            break;
                        }
                    }
                }
            }
        } else {
            break;
        }
    }

    if let Some(server) = &mut main_state.server {
        // if the server has made a move, call all clientes
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
    Ok(())
}
