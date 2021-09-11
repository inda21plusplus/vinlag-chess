use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// ONLY USED FOR RENDER
const WHITE_PICES: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
const BLACK_PICES: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
const ALPHABET: [&str; BOARD_SIZE] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const REVERSE_BOARD_ON_SWITCH: bool = false;
const BOARD_SIZE: usize = 8;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Vector2 {
    x: i8,
    y: i8,
}

struct Moveset {
    regular_moves: &'static [Vector2],
    inf_moves: &'static [Vector2],
}

/** GET MOVESET, WONT WORK FOR PAWN */
fn get_moveset(piece: Piece) -> Moveset {
    const DIAGONAL_MOVESET: &'static [Vector2; 4] = &[
        Vector2 { x: 1, y: 1 },
        Vector2 { x: -1, y: -1 },
        Vector2 { x: -1, y: 1 },
        Vector2 { x: 1, y: -1 },
    ];

    const HORIZONTAL_MOVESET: &'static [Vector2; 4] = &[
        Vector2 { x: 0, y: 1 },
        Vector2 { x: 0, y: -1 },
        Vector2 { x: -1, y: 0 },
        Vector2 { x: 1, y: 0 },
    ];

    const BOTH_MOVESET: &'static [Vector2; 8] = &[
        Vector2 { x: 0, y: 1 },
        Vector2 { x: 0, y: -1 },
        Vector2 { x: -1, y: 0 },
        Vector2 { x: 1, y: 0 },
        Vector2 { x: 1, y: 1 },
        Vector2 { x: -1, y: -1 },
        Vector2 { x: -1, y: 1 },
        Vector2 { x: 1, y: -1 },
    ];

    const KNIGHT_MOVESET: &'static [Vector2; 8] = &[
        Vector2 { x: 2, y: 1 },
        Vector2 { x: 1, y: 2 },
        Vector2 { x: -2, y: 1 },
        Vector2 { x: -1, y: 2 },
        Vector2 { x: 2, y: -1 },
        Vector2 { x: 1, y: -2 },
        Vector2 { x: -2, y: -1 },
        Vector2 { x: -1, y: -2 },
    ];

    const EMPTY_MOVESET: &'static [Vector2; 0] = &[];

    match piece {
        Piece::None => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
        Piece::Pawn => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
        Piece::Knight => Moveset {
            regular_moves: KNIGHT_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
        Piece::Bishop => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: DIAGONAL_MOVESET,
        },
        Piece::Rook => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: HORIZONTAL_MOVESET,
        },
        Piece::Queen => Moveset {
            regular_moves: EMPTY_MOVESET,
            inf_moves: BOTH_MOVESET,
        },
        Piece::King => Moveset {
            regular_moves: BOTH_MOVESET,
            inf_moves: EMPTY_MOVESET,
        },
    }
}

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
#[derive(Copy, Clone, PartialEq)]
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

/** 0,0 is the top left; 8,8 is the bottom right */
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Position {
    x: usize,
    y: usize,
}

struct Game {
    /** 0,0 is the top left; 8,8 is the bottom right */
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

fn clear_terminal_color(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    write!(&mut stdout, "{}", text).unwrap();
}

fn print_piece(text: &str, fg_color: Color, bg_color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(
            ColorSpec::new()
                .set_bold(true)
                .set_fg(Some(fg_color))
                .set_bg(Some(bg_color)),
        )
        .unwrap();

    write!(&mut stdout, " {}  ", text).unwrap();
}

fn render(game: &Game) {
    render_highlight(game, &HashSet::new(), Color::Red);
}

fn render_highlight(game: &Game, highlight: &HashSet<Position>, highlight_color: Color) {
    let is_inverted = REVERSE_BOARD_ON_SWITCH && !game.is_white_to_move;

    for y in 0..BOARD_SIZE {
        let display_y = if is_inverted { BOARD_SIZE - y } else { y + 1 };

        clear_terminal_color("");
        print!("{} ", BOARD_SIZE - y);
        for x in 0..BOARD_SIZE {
            let display_x = if is_inverted { BOARD_SIZE - x } else { x + 1 };

            let piece_data = game.board[display_x - 1][display_y - 1];

            let fg_color = if piece_data.is_white {
                Color::White
            } else {
                Color::Rgb(150, 150, 150) // Some(Color::Black)  //Some(Color::Rgb(150,150,150))
            };

            let bg_color = if highlight.contains(&Position { x, y }) {
                highlight_color
            } else {
                let is_dark_square = (display_y + display_x) % 2 == 0;

                if is_dark_square {
                    Color::Rgb(24, 26, 27) // Some(Color::Rgb(233,159,75)) //Some(Color::Rgb(24, 26, 27))
                } else {
                    Color::Rgb(49, 53, 55) // Some(Color::Rgb(171, 113, 59)) //Some(Color::Rgb(49, 53, 55))
                }
            };
            print_piece(
                get_char(piece_data.piece, piece_data.is_white),
                fg_color,
                bg_color,
            );
        }
        clear_terminal_color("\r\n");
    }
    print!("");
    for x in 0..BOARD_SIZE {
        print!("  {} ", ALPHABET[x]);
    }

    clear_terminal_color("\r\n\r\n");
}

fn parse_piece(input: char) -> Option<PieceData> {
    match input {
        'p' => Some(PieceData {
            piece: Piece::Pawn,
            is_white: false,
        }),
        'n' => Some(PieceData {
            piece: Piece::Knight,
            is_white: false,
        }),
        'b' => Some(PieceData {
            piece: Piece::Bishop,
            is_white: false,
        }),
        'r' => Some(PieceData {
            piece: Piece::Rook,
            is_white: false,
        }),
        'q' => Some(PieceData {
            piece: Piece::Queen,
            is_white: false,
        }),
        'k' => Some(PieceData {
            piece: Piece::King,
            is_white: false,
        }),

        'P' => Some(PieceData {
            piece: Piece::Pawn,
            is_white: true,
        }),
        'N' => Some(PieceData {
            piece: Piece::Knight,
            is_white: true,
        }),
        'B' => Some(PieceData {
            piece: Piece::Bishop,
            is_white: true,
        }),
        'R' => Some(PieceData {
            piece: Piece::Rook,
            is_white: true,
        }),
        'Q' => Some(PieceData {
            piece: Piece::Queen,
            is_white: true,
        }),
        'K' => Some(PieceData {
            piece: Piece::King,
            is_white: true,
        }),
        _ => None,
    }
}

/** Forsyth–Edwards Notation https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
start board for standard chess is rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
*/
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

/** Parse position in the e6 format */
fn parse_position(input: &str) -> Option<Position> {
    let real_input = input.to_lowercase();
    let chars: Vec<char> = real_input.chars().collect();

    // Invalid input length
    if chars.len() != 2 {
        return None;
    }

    const BOARD_X_INPUT: [char; BOARD_SIZE] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    const BOARD_Y_INPUT: [char; BOARD_SIZE] = ['8', '7', '6', '5', '4', '3', '2', '1'];

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
fn parse_move(input: &str) -> Option<(Position, Position)> {
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

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
enum MoveFlags {
    Invalid,
    Valid,
    Capture,
    WaitingForPromotion,
    BlackWon,
    WhiteWon,
    Tie,
}

fn get_position(pos: &Position, offset: &Vector2) -> Option<Position> {
    let new_position = Vector2 {
        x: pos.x as i8 + offset.x,
        y: pos.y as i8 + offset.y,
    };

    if new_position.x < 0
        || new_position.y < 0
        || new_position.x >= BOARD_SIZE as i8
        || new_position.y >= BOARD_SIZE as i8
    {
        return None;
    }

    Some(Position {
        x: new_position.x as usize,
        y: new_position.y as usize,
    })
}

fn generate_all_moves(game: &Game, piece_position: &Position) -> HashSet<Position> {
    let mut all_positions: HashSet<Position> = HashSet::new();

    let start_piece = game.board[piece_position.x][piece_position.y];
    if start_piece.piece == Piece::None {
        return all_positions;
    } else {
        if start_piece.piece == Piece::Pawn {
            //todo pawn logic
        } else {
            let moveset = get_moveset(start_piece.piece);

            // Goes though all jumps
            for r_move in moveset.regular_moves {
                let new_piece_position = get_position(piece_position, r_move);

                // check if valid
                if new_piece_position.is_none() {
                    continue;
                }

                let new_valid_position = new_piece_position.unwrap();
                if is_valid_capture(&game, &new_valid_position, start_piece.is_white) {
                    all_positions.insert(new_valid_position);
                }
            }

            // Goes though all inf move directions
            for i_move in moveset.inf_moves {
                let mut index = 0;
                loop {
                    index += 1;
                    let new_move = Vector2 {
                        x: i_move.x * index,
                        y: i_move.y * index,
                    };
                    println!("{} , {}", new_move.x, new_move.y);
                    let new_piece_position = get_position(piece_position, &new_move);

                    // check if valid
                    if new_piece_position.is_none() {
                        break;
                    }

                    let new_valid_position = new_piece_position.unwrap();
                    if is_valid_capture(&game, &new_valid_position, start_piece.is_white) {
                        all_positions.insert(new_valid_position);
                        // break if the piece is of another color, because the pawn cant ghost though pieces
                        let capture_piece = game.board[new_valid_position.x][new_valid_position.y];
                        if capture_piece.piece != Piece::None && (capture_piece.is_white != start_piece.is_white) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    all_positions
}

fn is_valid_capture(game: &Game, move_end: &Position, is_white: bool) -> bool {
    let capture_piece = game.board[move_end.x][move_end.y];

    // cant capture own colora
    if capture_piece.piece != Piece::None && (capture_piece.is_white == is_white) {
        return false;
    }

    true
}

/** basic move check */
fn is_valid_move(game: &Game, move_start: &Position, move_end: &Position) -> bool {
    if move_start.x >= BOARD_SIZE
        || move_start.y >= BOARD_SIZE
        || move_end.x >= BOARD_SIZE
        || move_end.y >= BOARD_SIZE
        || move_start == move_end
    {
        return false;
    }

    let start_piece = game.board[move_start.x][move_start.y];

    // start peice is invalid
    if (start_piece.is_white != game.is_white_to_move) || start_piece.piece == Piece::None {
        return false;
    }

    // cant capture own color
    if !is_valid_capture(&game, move_end, game.is_white_to_move) {
        return false;
    }

    true
}

fn move_piece(game: &mut Game, move_start: &Position, move_end: &Position) -> HashSet<MoveFlags> {
    let mut flags: HashSet<MoveFlags> = HashSet::new();

    // basic check first
    if !is_valid_move(game, move_start, move_end) {
        flags.insert(MoveFlags::Invalid);
        return flags;
    }

    // advanced check
    let all_valid_moves = generate_all_moves(&game, move_start);
    if !all_valid_moves.contains(move_end) {
        flags.insert(MoveFlags::Invalid);
        return flags;
    }

    let start_piece = game.board[move_start.x][move_start.y];
    let capture_piece = game.board[move_end.x][move_end.y];

    if capture_piece.piece != Piece::None {
        flags.insert(MoveFlags::Capture);
    }

    // MOVE LOGIC
    game.board[move_start.x][move_start.y] = PieceData {
        piece: Piece::None,
        is_white: false,
    };
    game.board[move_end.x][move_end.y] = start_piece;

    // update clock
    let half_move_clock = if capture_piece.piece == Piece::Pawn {
        0
    } else {
        game.half_move_clock + 1
    };

    let full_move_clock = if game.is_white_to_move {
        game.full_move_clock
    } else {
        game.full_move_clock + 1
    };

    game.half_move_clock = half_move_clock;
    game.full_move_clock = full_move_clock;
    game.is_white_to_move = !game.is_white_to_move;

    // 50 move rule
    if half_move_clock >= 50
        && !(flags.contains(&MoveFlags::BlackWon) || flags.contains(&MoveFlags::WhiteWon))
    {
        flags.insert(MoveFlags::Tie);
    }

    flags.insert(MoveFlags::Valid);
    flags
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn read_input() -> String {
    let mut text = String::new();
    std::io::stdin().read_line(&mut text).unwrap();
    trim_newline(&mut text);

    text
}

fn main() {
    //chcp 65001
    //std::process::Command::new("chcp 65001");
    //std::process::Command::new("clear");
    //std::process::Command::new("cls");
    println!("==================================");

    let mut game =
        get_board("rnbqkbnr/pppppppp/RNBQKBNR/8/8/8/8/8/ w KQkq - 0 1".to_string()).unwrap();

    loop {
        /*let mut highlight = HashSet::new();
        highlight.insert(Position { x: 0, y: 0 });
        game.board[0][0] = PieceData {
            piece: Piece::Pawn,
            is_white: false,
        };*/
        let moves = generate_all_moves(&game, &Position { x: 1, y: 2 });
        render_highlight(&game, &moves, Color::Red);
        //render(&game);

        //render_highlight(&game, &highlight, Color::Red);

        let input = read_input();
        let input_move = parse_move(&input);
        if input_move.is_none() {
            println!("Invalid Input");
            continue;
        }
        let parsed_move = input_move.unwrap();

        let move_data = move_piece(&mut game, &parsed_move.0, &parsed_move.1);
        if move_data.contains(&MoveFlags::Invalid) {
            println!("Invalid Move");
            continue;
        }
    }
}
