use crate::game_data::*;

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

/** Forsyth–Edwards Notation https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
start board for standard chess is rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
*/
pub fn print_board(game: &Game) -> Option<String> {
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
    if !game.white_castle.can_castle_king_side && !game.white_castle.can_castle_queen_side {
        cant_castle += 1;
    } else {
        if game.white_castle.can_castle_king_side {
            output.push('K');
        }
        if game.white_castle.can_castle_queen_side {
            output.push('Q');
        }
    }

    if !game.black_castle.can_castle_king_side && !game.black_castle.can_castle_queen_side {
        cant_castle += 1;
    } else {
        if game.black_castle.can_castle_king_side {
            output.push('k');
        }
        if game.black_castle.can_castle_queen_side {
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

    let mut white_castle = EMPTY_CASTLE;
    let mut black_castle = EMPTY_CASTLE;

    //todo fix
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

    let en_passant_position = None::<Position>; //TODO FIX split[3]

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
        white_castle: white_castle,
        black_castle: black_castle,
        is_white_to_move: is_white_to_move,
        en_passant_position: en_passant_position,
        half_move_clock: half_move_clock.unwrap(),
        full_move_clock: full_move_clock.unwrap(),
    };

    Some(game)
}
