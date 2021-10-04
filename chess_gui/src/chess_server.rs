use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chess_engine::game_data::{Gameboard, Piece, WinStatus};
use chess_engine::logic::get_game_state;
use chess_engine::parser;

use crate::{move_piece_with_state, MainState};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 128;

pub(crate) struct Server {
    listener: TcpListener,
    clients: Vec<TcpStream>,
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
}

pub(crate) fn start_server() -> Server {
    let listener = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    listener
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<(String, SocketAddr)>();

    return Server {
        listener,
        clients,
        tx,
        rx,
    };
}

fn get_state(gameboard: &Gameboard, win_status: WinStatus) -> Option<String> {
    let fen = &parser::get_board_fen(&gameboard.game);
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

pub(crate) fn server_loop(main_state: &mut MainState) -> Result<(), String> {
    if let Ok((mut socket, addr)) = main_state.server.listener.accept() {
        println!("Client {} connected", addr);

        let tx = main_state.server.tx.clone();
        main_state
            .server
            .clients
            .push(socket.try_clone().expect("failed to clone client"));

        // when a client joins, send them the board
        tx.send(("get:board".to_string(), addr))
            .expect("failed to send msg to rx");

        thread::spawn(move || loop {
            let mut buff = vec![0; MSG_SIZE];

            match socket.read_exact(&mut buff) {
                Ok(_) => {
                    // 0x3B is unicode for ';'
                    let msg = buff
                        .into_iter()
                        .take_while(|&x| (x != 0 && x != 0x3B))
                        .collect::<Vec<_>>();
                    let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                    println!("{}: {:?}", addr, msg);
                    tx.send((msg, addr)).expect("failed to send msg to rx");
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("closing connection with: {}", addr);
                    break;
                }
            }

            thread::sleep(std::time::Duration::from_millis(100)); // every connection gets a thread it looks
        });
    }

    loop {
        // do chess logic here
        if let Ok((msg, addr)) = main_state.server.rx.try_recv() {
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
                "get" => match &input[..] {
                    "board" => {
                        send_to_all = false;
                        let win_status = get_game_state(
                            &main_state.active_game.game,
                            &main_state.active_game.active_threats,
                            true,
                        );
                        let state = get_state(&main_state.active_game.game, win_status);
                        if state.is_some() {
                            send_msg = state.unwrap();
                        } else {
                            send_msg = "err:state_invalid".to_string()
                        }
                    }
                    "spectatorcount" => {
                        send_to_all = false;
                        let mut result = "spectatorcount:".to_string();
                        result.push_str(&(main_state.server.clients.len() - 1).to_string());
                        send_msg = result;
                    }
                    _ => {}
                },
                "move" => {
                    if input.len() != 5 {
                        continue;
                    };
                    println!("Preformed move 1");

                    let (move_from, move_to) = match parser::parse_move(&input[0..4].to_string()) {
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

                    println!("Preformed move 2");
                    let win_status =
                        move_piece_with_state(main_state, move_from, move_to, promotion);
                    let state = get_state(&main_state.active_game.game, win_status);
                    if state.is_some() {
                        send_msg = state.unwrap();
                    } else {
                        send_msg = "err:state_invalid".to_string()
                    }
                }
                _ => {
                    println!("Normal msg");
                }
            }

            send_msg.push(';');
            let mut buff = send_msg.into_bytes();
            buff.resize(MSG_SIZE, 0);

            if send_to_all {
                for client in &mut main_state.server.clients {
                    let _write_error = client.write_all(&buff);
                }
            } else {
                for client in &mut main_state.server.clients {
                    let client_addr = client.peer_addr();
                    if client_addr.is_ok() && client_addr.unwrap() == addr {
                        let _write_error = client.write_all(&buff);
                        break;
                    }
                }
            }
        } else {
            break;
        }
    }

    // if the server has made a move, call all clientes
    if main_state.active_game.penging_send {
        main_state.active_game.penging_send = false;
        let win_status = get_game_state(
            &main_state.active_game.game,
            &main_state.active_game.active_threats,
            true,
        );
        let state = get_state(&main_state.active_game.game, win_status);
        if state.is_some() {
            let mut send_msg = state.unwrap();
            send_msg.push(';');
            let mut buff = send_msg.into_bytes();
            buff.resize(MSG_SIZE, 0);
            for client in &mut main_state.server.clients {
                let _write_error = client.write_all(&buff);
            }
        }
    }

    Ok(())
}
