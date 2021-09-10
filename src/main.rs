use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const BLACK_PICES: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
const WHITE_PICES: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
const BOARD_SIZE: usize = 8;
const ALPHABET: [&str; BOARD_SIZE] = ["a", "b", "c", "d", "e", "f", "g", "h"];

fn get_char(piece: Piece, is_white: bool) -> &'static str {
    let set = if is_white { WHITE_PICES } else { BLACK_PICES };

    match piece {
        Piece::None => " ",
        Piece::Pawn => set[0],
        Piece::Knight => set[1],
        Piece::Bishop => set[2],
        Piece::Rook => set[3],
        Piece::Queen => set[4],
        Piece::King => set[5],
    }
}
#[derive(Copy, Clone)]
enum Piece {
    None,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone)]
struct PieceData {
    piece: Piece,
    is_white: bool,
}

struct Castle {
    can_castle_king_side: bool,
    can_castle_queen_side: bool,
}

struct Position {
    x: u8,
    y: u8,
}

struct Game {
    board: [[PieceData; BOARD_SIZE]; BOARD_SIZE],

    white_castle: Castle,
    black_castle: Castle,

    is_white_to_move: bool,

    /**
    This is recorded regardless of whether there is a pawn in position to make an en passant capture.
    */
    en_passant_position: Option<Position>,

    /**
    Halfmove clock: The number of halfmoves since the last capture or pawn advance,
    used for the fifty-move rule. https://en.wikipedia.org/wiki/Fifty-move_rule
    */
    half_move_clock: u16,
    /**
    Fullmove number: The number of the full move. It starts at 1, and is incremented after Black's move.
    */
    full_move_clock: u16,
}

fn clear_terminal_color(text: &str) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    write!(&mut stdout, "{}", text)
}

fn change_terminal_colors(text: &str, is_white: bool, is_dark: bool) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(
        ColorSpec::new()
            .set_bold(true)
            .set_fg(if is_white {
                Some(Color::White)
            } else {
                Some(Color::Rgb(150, 150, 150)) // Some(Color::Black)  //Some(Color::Rgb(150,150,150))
            })
            .set_bg(if is_dark {
                Some(Color::Rgb(24, 26, 27)) // Some(Color::Rgb(233,159,75)) //Some(Color::Rgb(24, 26, 27))
            } else {
                Some(Color::Rgb(49, 53, 55)) // Some(Color::Rgb(171, 113, 59)) //Some(Color::Rgb(49, 53, 55))
            }),
    )?;

    write!(&mut stdout, " {}  ", text)
}

fn render(game: &Game) {
    let is_inverted = !game.is_white_to_move;

    for y in 0..BOARD_SIZE {
        let display_y = if is_inverted {y+1} else { BOARD_SIZE - y};

        clear_terminal_color("").unwrap();
        print!("{} ", display_y);
        for x in 0..BOARD_SIZE {
            let display_x = if is_inverted {x+1} else { BOARD_SIZE - x};

            let piece_data = game.board[display_x-1][display_y-1];

            change_terminal_colors(
                get_char(piece_data.piece, piece_data.is_white),
                piece_data.is_white,
                (display_y + display_x) % 2 == 0,
            )
            .unwrap();
        }
        clear_terminal_color("\r\n").unwrap();
    }
    print!("");
    for x in 0..BOARD_SIZE {
        print!("  {} ", ALPHABET[x]);
    }

    clear_terminal_color("").unwrap();
}

fn parse_piece(input: char) -> Option<PieceData> {
    match input {
        'p' => Some(PieceData {
            piece: Piece::Pawn,
            is_white: true,
        }),
        'n' => Some(PieceData {
            piece: Piece::Knight,
            is_white: true,
        }),
        'b' => Some(PieceData {
            piece: Piece::Bishop,
            is_white: true,
        }),
        'r' => Some(PieceData {
            piece: Piece::Rook,
            is_white: true,
        }),
        'q' => Some(PieceData {
            piece: Piece::Queen,
            is_white: true,
        }),
        'k' => Some(PieceData {
            piece: Piece::King,
            is_white: true,
        }),

        'P' => Some(PieceData {
            piece: Piece::Pawn,
            is_white: false,
        }),
        'N' => Some(PieceData {
            piece: Piece::Knight,
            is_white: false,
        }),
        'B' => Some(PieceData {
            piece: Piece::Bishop,
            is_white: false,
        }),
        'R' => Some(PieceData {
            piece: Piece::Rook,
            is_white: false,
        }),
        'Q' => Some(PieceData {
            piece: Piece::Queen,
            is_white: false,
        }),
        'K' => Some(PieceData {
            piece: Piece::King,
            is_white: false,
        }),
        _ => None,
    }
}

//https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
fn get_board(fen_string: String) -> Option<Game> {
    let split: Vec<String> = fen_string
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // cant parse, invalid format
    if split.len() != 6 {
        return None;
    }

    // get board
    let mut board = [[PieceData {
        piece: Piece::None,
        is_white: false,
    }; BOARD_SIZE]; BOARD_SIZE];
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

    //invalid input
    if casle_chars.len() != 4 {
        return None;
    }

    let white_castle = Castle {
        can_castle_king_side: casle_chars[0] != '-',
        can_castle_queen_side: casle_chars[1] != '-',
    };
    let black_castle = Castle {
        can_castle_king_side: casle_chars[2] != '-',
        can_castle_queen_side: casle_chars[3] != '-',
    };

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

fn main() {
    //chcp 65001
    let mut game = get_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()).unwrap();
    render(&game);
}
