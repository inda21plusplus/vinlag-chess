use std::collections::HashMap;

use crate::game_data::*;
pub const STANDARD_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn init_game_board(fen: String) -> Option<Gameboard> {
    let game = match get_board(fen) {
        Some(g) => g,
        None => return None,
    };

    let mut map = HashMap::new();
    // inserts the standard board
    map.insert(
        match get_board_fen(&game) {
            Some(g) => g,
            None => return None,
        },
        1,
    );

    return Some(Gameboard {
        game: game,
        same_board: map,
    });
}

#[test]
fn parse_test_1() {
    let pos_unchecked = parse_position("a1");
    assert!(pos_unchecked.is_some());
    assert_eq!(
        pos_unchecked.unwrap(),
        Position {
            x: 0,
            y: BOARD_SIZE - 1
        }
    );
}

#[test]
fn parse_test_2() {
    let pos_unchecked = parse_position("h8");
    assert!(pos_unchecked.is_some());
    assert_eq!(
        pos_unchecked.unwrap(),
        Position {
            x: BOARD_SIZE - 1,
            y: 0
        }
    );
}

#[test]
fn parse_test_invalid() {
    assert!(parse_position("h9").is_none());
    assert!(parse_position("a0").is_none());
    assert!(parse_position("a").is_none());
    assert!(parse_position("0").is_none());
    assert!(parse_position("--").is_none());
    assert!(parse_position("").is_none());
    assert!(parse_position(" ").is_none());
    assert!(parse_position("\n").is_none());
}

#[test]
fn parse_test_valid() {
    assert!(parse_position("h7").is_some());
    assert!(parse_position("h2").is_some());
    assert!(parse_position("e2").is_some());
    assert!(parse_position("c1").is_some());
    assert!(parse_position("f4").is_some());

    assert!(parse_position("F4").is_some());
    assert!(parse_position("A1").is_some());
    assert!(parse_position("B2").is_some());
    assert!(parse_position("C8").is_some());
}

#[test]
fn fen_test_no_castle() {
    let str = "rnbqk2r/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQK2R w KQkq - 2 5";
    let board = get_board(str.to_string());
    assert!(board.is_some());
    let board_string = get_fen(&board.unwrap());
    assert!(board_string.is_some());
    let valid_board_string = board_string.unwrap();
    assert_eq!(str, valid_board_string);
}

#[test]
fn fen_test_one_castle() {
    let str = "rnbqk2r/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 b kq - 3 5";
    let board = get_board(str.to_string());
    assert!(board.is_some());
    let board_string = get_fen(&board.unwrap());
    assert!(board_string.is_some());
    let valid_board_string = board_string.unwrap();
    assert_eq!(str, valid_board_string);
}

#[test]
fn fen_test_both_castle() {
    let str = "rnbq1rk1/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 w - - 4 6";
    let board = get_board(str.to_string());
    assert!(board.is_some());
    let board_string = get_fen(&board.unwrap());
    assert!(board_string.is_some());
    let valid_board_string = board_string.unwrap();
    assert_eq!(str, valid_board_string);
}

/** Parse position in the e6 format */
fn parse_position(input: &str) -> Option<Position> {
    let real_input = input.to_lowercase();
    let chars: Vec<char> = real_input.chars().collect();

    // Invalid input length
    if chars.len() != 2 {
        return None;
    }

    let x = BOARD_X_INPUT.iter().position(|&c| c == chars[0]);
    let y = BOARD_Y_INPUT.iter().position(|&c| c == chars[1]);

    // invalid input chars
    if x.is_none() || y.is_none() {
        return None;
    }

    return Some(Position {
        x: x.unwrap(),
        y: y.unwrap(),
    });
}

pub fn get_position(pos: Position) -> String {
    let mut str = String::new();
    str.push(BOARD_X_INPUT[pos.x]);
    str.push(BOARD_Y_INPUT[pos.y]);
    return str;
}

pub fn get_move(move_from: Position, move_to: Position) -> String {
    let str1 = get_position(move_from);
    let str2 = get_position(move_to);
    return format!("{}{}", str1, str2);
}

/** Parse move in e6e3 format, result as from -> to */
pub fn parse_move(input: &str) -> Option<(Position, Position)> {
    if input.len() != 4 {
        return None;
    }

    let split_input = input.split_at(2);
    let move_start = parse_position(split_input.0);
    let move_end = parse_position(split_input.1);
    if move_start.is_none() || move_end.is_none() {
        return None;
    }

    Some((move_start.unwrap(), move_end.unwrap()))
}

const CHAR_TO_PIECE_MAP: [(char, PieceData); 12] = [
    (
        'p',
        PieceData {
            piece: Piece::Pawn,
            is_white: false,
        },
    ),
    (
        'n',
        PieceData {
            piece: Piece::Knight,
            is_white: false,
        },
    ),
    (
        'b',
        PieceData {
            piece: Piece::Bishop,
            is_white: false,
        },
    ),
    (
        'r',
        PieceData {
            piece: Piece::Rook,
            is_white: false,
        },
    ),
    (
        'q',
        PieceData {
            piece: Piece::Queen,
            is_white: false,
        },
    ),
    (
        'k',
        PieceData {
            piece: Piece::King,
            is_white: false,
        },
    ),
    (
        'P',
        PieceData {
            piece: Piece::Pawn,
            is_white: true,
        },
    ),
    (
        'N',
        PieceData {
            piece: Piece::Knight,
            is_white: true,
        },
    ),
    (
        'B',
        PieceData {
            piece: Piece::Bishop,
            is_white: true,
        },
    ),
    (
        'R',
        PieceData {
            piece: Piece::Rook,
            is_white: true,
        },
    ),
    (
        'Q',
        PieceData {
            piece: Piece::Queen,
            is_white: true,
        },
    ),
    (
        'K',
        PieceData {
            piece: Piece::King,
            is_white: true,
        },
    ),
];

fn get_piece(piece_data: PieceData) -> Option<char> {
    for value in CHAR_TO_PIECE_MAP {
        if value.1 == piece_data {
            return Some(value.0);
        }
    }
    None
}

fn parse_piece(input: char) -> Option<PieceData> {
    for value in CHAR_TO_PIECE_MAP {
        if value.0 == input {
            return Some(value.1);
        }
    }
    None
}

pub fn get_board_fen(game: &Game) -> Option<String> {
    let mut output: String = String::new();
    // generate board
    for y in 0..BOARD_SIZE {
        let mut last_piece: u8 = 0;
        for x in 0..BOARD_SIZE {
            let piece_data = game.board[x][y];
            if piece_data.piece == Piece::None {
                last_piece += 1;
            } else {
                if last_piece != 0 {
                    output.push((last_piece + 48u8) as char); // ascci int to char
                }
                match get_piece(piece_data) {
                    Some(s_char) => output.push(s_char),
                    None => return None,
                }
                last_piece = 0
            }
        }
        if last_piece != 0 {
            output.push((last_piece + 48u8) as char);
        }
        if y != BOARD_SIZE - 1 {
            output.push('/');
        }
    }

    // white/black to move
    output.push(' ');
    output.push(if game.is_white_to_move { 'w' } else { 'b' });
    output.push(' ');

    let mut cant_castle = 0;

    // casteling
    if !game.castle[0].can_castle_king_side && !game.castle[0].can_castle_queen_side {
        cant_castle += 1;
    } else {
        if game.castle[1].can_castle_king_side {
            output.push('K');
        }
        if game.castle[1].can_castle_queen_side {
            output.push('Q');
        }
    }

    if !game.castle[1].can_castle_king_side && !game.castle[1].can_castle_queen_side {
        cant_castle += 1;
    } else {
        if game.castle[1].can_castle_king_side {
            output.push('k');
        }
        if game.castle[1].can_castle_queen_side {
            output.push('q');
        }
    }

    if cant_castle == 2 {
        output.push('-');
        output.push(' ');
    }

    if !output.ends_with(' ') {
        output.push(' ');
    }

    if game.en_passant_position.is_some() {
        let en_passant_position = game.en_passant_position.unwrap();
        output.push(BOARD_X_INPUT[en_passant_position.x]);
        output.push(BOARD_Y_INPUT[en_passant_position.y]);
    } else {
        output.push('-');
    }

    return Some(output);
}

/** Forsyth–Edwards Notation https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
start board for standard chess is rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
*/
pub fn get_fen(game: &Game) -> Option<String> {
    let mut output = match get_board_fen(game) {
        Some(fen) => fen,
        None => return None,
    };
    output.push(' ');

    output += &game.half_move_clock.to_string();
    output.push(' ');
    output += &game.full_move_clock.to_string();

    Some(output)
}

/** Forsyth–Edwards Notation https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
start board for standard chess is rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
*/
pub fn get_board(fen_string: String) -> Option<Game> {
    let split: Vec<String> = fen_string
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // cant parse, invalid format
    if split.len() != 6 {
        return None;
    }

    // get board
    let mut board = [[EMPTY_PEICE; BOARD_SIZE]; BOARD_SIZE];
    let mut board_x = 0usize;
    let mut board_y = 0usize;
    for char in split[0].chars() {
        if char == '/' || board_x >= BOARD_SIZE {
            board_y += 1;
            board_x = 0;
            continue;
        }
        if board_y >= BOARD_SIZE {
            // this should not happend, invalid input?
            break;
        }

        let piece = parse_piece(char);

        if piece.is_none() {
            // is number
            let number: Option<u32> = char.to_digit(10);
            if number.is_none() {
                // invalid input
                return None;
            }
            board_x += number.unwrap() as usize;
        } else {
            // is piece
            board[board_x][board_y] = piece.unwrap();
            board_x += 1;
        }
    }

    // who to move
    let is_white_to_move = split[1] == "w";

    //castle
    let casle_chars: Vec<char> = split[2].chars().collect(); //.chars();

    let mut white_castle = Castle {
        can_castle_king_side: false,
        can_castle_queen_side: false,
        queen_side_rook: Position {
            x: 0,
            y: BOARD_SIZE - 1,
        },
        king_side_rook: Position {
            x: BOARD_SIZE - 1,
            y: BOARD_SIZE - 1,
        },
    };
    let mut black_castle = Castle {
        can_castle_king_side: false,
        can_castle_queen_side: false,
        queen_side_rook: Position { x: 0, y: 0 },
        king_side_rook: Position {
            x: BOARD_SIZE - 1,
            y: 0,
        },
    };

    for casle_char in casle_chars {
        match casle_char {
            'K' => {
                white_castle.can_castle_king_side = true;
            }
            'k' => {
                black_castle.can_castle_king_side = true;
            }
            'Q' => {
                white_castle.can_castle_queen_side = true;
            }
            'q' => {
                black_castle.can_castle_queen_side = true;
            }
            _ => {}
        };
    }

    let en_passant_position = parse_position(&split[3]);

    let half_move_clock = split[4].parse::<u16>();
    if half_move_clock.is_err() {
        // invalid input
        return None;
    }

    let full_move_clock = split[5].parse::<u16>();
    if full_move_clock.is_err() {
        // invalid input
        return None;
    }

    let game = Game {
        board: board,
        castle: [white_castle, black_castle],
        is_white_to_move: is_white_to_move,
        en_passant_position: en_passant_position,
        half_move_clock: half_move_clock.unwrap(),
        full_move_clock: full_move_clock.unwrap(),
    };

    Some(game)
}
