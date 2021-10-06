use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::TcpStream;

use chess_engine::game_data::WinStatus;
use chess_engine::parser;

use crate::{get_loaded_game, MainState};

const LOCAL: &str = "127.0.0.1:1337";

pub(crate) struct Client {
    stream: TcpStream,
    pub(crate) is_player: bool,
    // tx: Sender<String>,
    // rx: Receiver<String>,
}

pub(crate) fn start_client() -> Option<Client> {
    if let Ok(mut stream) = TcpStream::connect(LOCAL) {
        stream
            .set_nonblocking(true)
            .expect("failed to initiate non-blocking");

        if stream.write_all(b"init:;").is_ok() {
            //    let (tx, rx) = mpsc::channel::<(String, SocketAddr)>();

            return Some(Client {
                stream,
                is_player: true,
                //  tx,
                //  rx,
            });
        }
    } else {
        println!("Failed to connect to {}", LOCAL);
    }

    return None;
}

pub(crate) fn client_loop(main_state: &mut MainState) {
    let mut is_connected = true;
    let mut handle_msg: Vec<String> = Vec::new();

    if let Some(client) = &mut main_state.client {
        let read_buffer = BufReader::new(&mut client.stream);
        let mut split = read_buffer.split(b';');
        while let Some(read) = split.next() {
            match read {
                Ok(msg) => {
                    let return_message = String::from_utf8_lossy(&msg);
                    println!("message recv {:?}", return_message);
                    handle_msg.push(return_message.to_string());
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => {
                    println!("connection with server was severed");
                    is_connected = false;
                    break;
                }
            }
        }
    }

    if is_connected {
        if let Some((pending_move_from, pending_move_to, promotion)) =
            main_state.active_game.pending_move
        {
            let mut send_msg = "move:".to_string();
            send_msg.push_str(&parser::get_move(pending_move_from, pending_move_to));
            send_msg.push(match promotion {
                chess_engine::game_data::Piece::Bishop => 'b',
                chess_engine::game_data::Piece::Rook => 'r',
                chess_engine::game_data::Piece::Queen => 'q',
                chess_engine::game_data::Piece::None => '-',
                chess_engine::game_data::Piece::Pawn => '-',
                chess_engine::game_data::Piece::Knight => 'n',
                chess_engine::game_data::Piece::King => '-',
            });
            send_msg.push(';');

            let buff = send_msg.into_bytes();
            if let Some(client) = &mut main_state.client {
                let write_error = client.stream.write_all(&buff);
                if write_error.is_ok() {
                    main_state.active_game.pending_move = None;
                }
            }
        }

        for msg in handle_msg {
            let split: Vec<String> = msg.split(":").map(|s| s.to_string()).collect();
            if split.len() != 2 {
                return;
            }

            let action = &split[0];
            let input = &split[1];

            match &action[..] {
                "playertype" => {
                    if let Some(client) = &mut main_state.client {
                        match input.chars().nth(0) {
                            Some('p') => client.is_player = true,
                            Some('s') => client.is_player = false,
                            None => (),
                            _ => (),
                        }
                    }
                }
                "board" => {
                    if let Some((game, threats)) = get_loaded_game(input.to_string()) {
                        main_state.active_game.win_status = WinStatus::Nothing;
                        main_state.active_game.game = game;
                        main_state.active_game.active_threats = threats;
                    }
                }
                "end" => {
                    let win_status = match input.chars().nth(0) {
                        Some('w') => WinStatus::WhiteWon,
                        Some('b') => WinStatus::BlackWon,
                        Some('-') => WinStatus::Tie,
                        None => WinStatus::Nothing,
                        _ => WinStatus::Nothing,
                    };

                    main_state.active_game.win_status = win_status;

                    if let Some((game, threats)) = get_loaded_game(input[1..].to_string()) {
                        main_state.active_game.game = game;
                        main_state.active_game.active_threats = threats;
                    }
                }
                _ => (),
            }
        }
    } else {
        main_state.client = None;
    }
}
