use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
    option,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// ONLY USED FOR RENDER
const WHITE_PICES: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
const BLACK_PICES: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
const ALPHABET: [&str; BOARD_SIZE] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const REVERSE_BOARD_ON_SWITCH: bool = false;
const BOARD_SIZE: usize = 8;

// shorthand
const BLACK_SPAWN: usize = 0;
const WHITE_SPAWN: usize = BOARD_SIZE - 1;
const WHITE_PAWN_Y: usize = BOARD_SIZE - 2;
const BLACK_PAWN_Y: usize = 1;
const EMPTY_PEICE: PieceData = PieceData {
    piece: Piece::None,
    is_white: false,
};

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
    InvalidWaitingForPromotion,
    InvalidRevealKing,
    Valid,
    Capture,
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

fn is_square_none(game: &Game, piece_position: &Position) -> bool {
    return game.board[piece_position.x][piece_position.y].piece == Piece::None;
}

fn is_square_color(game: &Game, piece_position: &Position, is_white: bool) -> bool {
    let piece_data = game.board[piece_position.x][piece_position.y];
    return piece_data.piece != Piece::None && piece_data.is_white == is_white;
}

/** Excludes casteling to not make an while true loop while */
fn generate_all_moves(game: &Game, piece_position: &Position) -> HashSet<Position> {
    let mut all_positions: HashSet<Position> = HashSet::new();

    let start_piece = game.board[piece_position.x][piece_position.y];
    if start_piece.piece == Piece::None {
        return all_positions;
    } else {
        // special case for pawns because they have so many rules
        if start_piece.piece == Piece::Pawn {
            let start_position = if start_piece.is_white {
                WHITE_PAWN_Y
            } else {
                BLACK_PAWN_Y
            };

            let move_direction: i8 = if start_piece.is_white { -1 } else { 1 };

            // if the pawn has not moved
            if start_position == piece_position.y {
                let pos = get_position(
                    piece_position,
                    &Vector2 {
                        x: 0,
                        y: move_direction * 2,
                    },
                )
                .unwrap();
                if is_square_none(game, &pos) {
                    all_positions.insert(pos);
                }
            };

            // handle standard advance
            let pos_advance = get_position(
                piece_position,
                &Vector2 {
                    x: 0,
                    y: move_direction,
                },
            )
            .unwrap();

            if is_square_none(game, &pos_advance) {
                all_positions.insert(pos_advance);
            }

            // handle diagonal moves
            let pawn_movelist: &[Vector2; 2] = &[
                Vector2 {
                    x: -1,
                    y: move_direction,
                },
                Vector2 {
                    x: 1,
                    y: move_direction,
                },
            ];

            for new_move in pawn_movelist {
                let new_valid_position = match get_position(piece_position, &new_move) {
                    Some(pos) => pos,
                    None => continue,
                };

                // en passant is avalible
                if game.en_passant_position.is_some()
                    && new_valid_position == game.en_passant_position.unwrap()
                {
                    all_positions.insert(new_valid_position);
                } else if is_square_color(game, &new_valid_position, !start_piece.is_white) {
                    all_positions.insert(new_valid_position);
                }
            }
        } else {
            let moveset = get_moveset(start_piece.piece);

            // Goes though all jumps
            for r_move in moveset.regular_moves {
                let new_valid_position = match get_position(piece_position, &r_move) {
                    Some(pos) => pos,
                    None => continue,
                };

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

                    let new_valid_position = match get_position(piece_position, &new_move) {
                        Some(pos) => pos,
                        None => break,
                    };

                    if is_valid_capture(&game, &new_valid_position, start_piece.is_white) {
                        all_positions.insert(new_valid_position);
                        // break if the piece is of another color, because the pawn cant ghost though pieces
                        let capture_piece = game.board[new_valid_position.x][new_valid_position.y];
                        if capture_piece.piece != Piece::None
                            && (capture_piece.is_white != start_piece.is_white)
                        {
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
    // cant capture own color
    if is_square_color(game, move_end, is_white) {
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

//will fail if some gamemode spawns pawns at the beginning
fn get_promotion_pawn(game: &Game) -> Option<Position> {
    const PAWN_CHECKS: [usize; 2] = [BLACK_SPAWN, WHITE_SPAWN];

    for y in PAWN_CHECKS {
        for x in 0..BOARD_SIZE {
            let piece_data = game.board[x][y];
            if piece_data.piece == Piece::Pawn {
                return Some(Position { x: x, y: y });
            }
        }
    }

    None
}

fn promote_pawn(game: &mut Game, promotion: Piece) -> bool {
    // check for invalid input
    if promotion != Piece::Bishop
        && promotion != Piece::Knight
        && promotion != Piece::Rook
        && promotion != Piece::Queen
    {
        return false;
    }

    // no pawn to promote
    let position = match get_promotion_pawn(game) {
        Some(pos) => pos,
        None => return false,
    };

    game.board[position.x][position.y].piece = promotion;
    return true;
}

/** Includes all threat positions generate by that team */
fn generate_all_threats(game: &Game, is_white: bool) -> HashSet<Position> {
    let mut all_threats: HashSet<Position> = HashSet::new();
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let position = Position { x, y };
            if is_square_color(game, &position, is_white) {
                let piece_threads = generate_all_moves(game, &position);
                for t_pos in piece_threads {
                    all_threats.insert(t_pos);
                }
            }
        }
    }

    all_threats
}

fn find_king(game: &Game, is_white: bool) -> Option<Position> {
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let piece_data = game.board[x][y];
            if piece_data.piece == Piece::King && piece_data.is_white == is_white {
                return Some(Position { x, y });
            }
        }
    }
    None
}

/**Returns true if castle is successful */
fn try_castle(game: &mut Game, move_start: &Position, move_end: &Position) -> bool {
    let is_white = game.is_white_to_move;
    let start_piece = game.board[move_start.x][move_start.y];

    // castle
    let castle_status = if is_white {
        &game.white_castle
    } else {
        &game.white_castle
    };

    // be aware that you can trick this by adding a second rook because
    // it only keeps track of the first rook on the left or right side
    if start_piece.piece == Piece::King
        && (castle_status.can_castle_king_side || castle_status.can_castle_queen_side)
    {
        let spawn_y = if is_white { WHITE_SPAWN } else { BLACK_SPAWN };

        // right y pos
        if spawn_y == move_start.y {
            let other_threats = generate_all_threats(game, !is_white);

            // king cant castle if checked
            if !other_threats.contains(move_start) {
                let is_king_side = move_end.x > move_start.x;

                // if can castle
                if if is_king_side {
                    castle_status.can_castle_king_side
                } else {
                    castle_status.can_castle_queen_side
                } {
                    // what direction is king side
                    let offset = if is_king_side { 1 } else { -1 };

                    for index in 1..BOARD_SIZE {
                        let new_valid_position = match get_position(
                            move_start,
                            &Vector2 {
                                x: (index as i8) * offset,
                                y: 0,
                            },
                        ) {
                            Some(pos) => pos,
                            None => break,
                        };

                        // the king cant be checked on his way over to the rook,
                        // this also checks the rook square as an added benift
                        if other_threats.contains(&new_valid_position) {
                            break;
                        }

                        let piece_data = game.board[new_valid_position.x][new_valid_position.y];

                        // found rook
                        if piece_data.is_white == is_white && piece_data.piece == Piece::Rook {
                            let new_rook_position = match get_position(
                                move_start,
                                &Vector2 {
                                    x: (index as i8 - 2) * offset,
                                    y: 0,
                                },
                            ) {
                                Some(pos) => pos,
                                None => break,
                            };

                            // if rook position == end move + offset
                            if new_valid_position.x as i8 == move_end.x as i8 + offset {
                                // moves rook
                                game.board[new_rook_position.x][new_rook_position.y] = piece_data;
                                // moves king
                                game.board[move_end.x][move_end.y] =
                                    game.board[move_start.x][move_start.y];

                                // clears old rook
                                game.board[new_valid_position.x][new_valid_position.y] =
                                    EMPTY_PEICE;
                                // clears old king
                                game.board[move_start.x][move_start.y] = EMPTY_PEICE;

                                let empty_castle = Castle {
                                    can_castle_king_side: false,
                                    can_castle_queen_side: false,
                                };

                                // removes the has castled
                                if is_white {
                                    game.white_castle = empty_castle;
                                } else {
                                    game.black_castle = empty_castle;
                                }

                                return true;
                            }
                        } else if piece_data.piece != Piece::None {
                            // cant jump over pieces
                            break;
                        }
                    }
                }
            }
        }
    }
    return false;
}

fn move_piece(
    game: &mut Game,
    move_start: &Position,
    move_end: &Position,
    auto_promote: bool,
) -> HashSet<MoveFlags> {
    let mut flags: HashSet<MoveFlags> = HashSet::new();
    let is_white = game.is_white_to_move;

    if get_promotion_pawn(game).is_some() {
        flags.insert(MoveFlags::Invalid);
        flags.insert(MoveFlags::InvalidWaitingForPromotion);
        return flags;
    }

    // basic check first
    if !is_valid_move(game, move_start, move_end) {
        flags.insert(MoveFlags::Invalid);
        return flags;
    }

    let start_piece = game.board[move_start.x][move_start.y];

    let done_castle = try_castle(game, move_start, move_end);

    let mut half_move_clock = game.half_move_clock + 1;
    let mut en_passant_position: Option<Position> = None;

    // have already done casteling, ignore the regular moves
    if !done_castle {
        // advanced check
        let all_valid_moves = generate_all_moves(&game, move_start);
        if !all_valid_moves.contains(move_end) {
            flags.insert(MoveFlags::Invalid);
            return flags;
        }

        let capture_piece = game.board[move_end.x][move_end.y];

        // moves the piece
        game.board[move_start.x][move_start.y] = EMPTY_PEICE;
        game.board[move_end.x][move_end.y] = start_piece;

        let other_threats = generate_all_threats(game, !is_white);
        let king_position = find_king(&game, is_white).unwrap();

        // move is invalid because it causes the king to be in check
        if other_threats.contains(&king_position) {
            // undo move
            game.board[move_start.x][move_start.y] = start_piece;
            game.board[move_end.x][move_end.y] = capture_piece;

            flags.insert(MoveFlags::InvalidRevealKing);
            flags.insert(MoveFlags::Invalid);
            return flags;
        }

        // en passant setup
        if start_piece.piece == Piece::Pawn {
            let pawn_spawn = if start_piece.is_white {
                WHITE_PAWN_Y
            } else {
                BLACK_PAWN_Y
            };

            if move_start.y == pawn_spawn && move_end.y == move_start.y + 2usize {
                en_passant_position = get_position(move_start, &Vector2 { x: 0, y: 1 });
            }
        } else if start_piece.piece == Piece::Rook {
            //TODO random chess add start position

            //removes the castling avalibility
            let spawn = if is_white { WHITE_SPAWN } else { BLACK_SPAWN };
            if move_start.y == spawn {
                if move_start.x == 0 {
                    if is_white {
                        game.white_castle.can_castle_queen_side = false;
                    } else {
                        game.black_castle.can_castle_queen_side = false;
                    }
                } else if move_start.x == BOARD_SIZE-1 {
                    if is_white {
                        game.white_castle.can_castle_king_side = false;
                    } else {
                        game.black_castle.can_castle_king_side = false;
                    }
                }
            }
        }

        if auto_promote {
            promote_pawn(game, Piece::Queen);
        }

        if capture_piece.piece != Piece::None {
            flags.insert(MoveFlags::Capture);

            if capture_piece.piece == Piece::Pawn {
                half_move_clock = 0;
            }
        }
    }

    if start_piece.piece == Piece::King {
        let castle = Castle {
            can_castle_king_side: false,
            can_castle_queen_side: false,
        };
        if is_white {
            game.white_castle = castle;
        } else {
            game.black_castle = castle;
        }
    }

    // todo checkmate or tie

    // update clock

    let full_move_clock = if is_white {
        game.full_move_clock
    } else {
        game.full_move_clock + 1
    };

    game.half_move_clock = half_move_clock;
    game.full_move_clock = full_move_clock;
    game.is_white_to_move = !is_white;
    game.en_passant_position = en_passant_position;

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
        get_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1".to_string()).unwrap();

    loop {
        /*let mut highlight = HashSet::new();
        highlight.insert(Position { x: 0, y: 0 });
        game.board[0][0] = PieceData {
            piece: Piece::Pawn,
            is_white: false,
        };*/
        /*

        let mut offset_index = 0;
        loop {
            let pos = Position {
                x: offset_index,
                y: WHITE_SPAWN,
            };
            let mut moves = generate_all_moves(&game, &pos);
            moves.insert(pos);

            render_highlight(&game, &moves, Color::Red);
            let input = read_input();
            offset_index += 1;
        }

        */

        render(&game);

        //let highlight = generate_all_threats(&game,true);
        //render_highlight(&game, &highlight, Color::Red);

        let input = read_input();
        let input_move = parse_move(&input);
        if input_move.is_none() {
            println!("Invalid Input");
            continue;
        }
        let parsed_move = input_move.unwrap();

        let move_data = move_piece(&mut game, &parsed_move.0, &parsed_move.1, true);
        if move_data.contains(&MoveFlags::Invalid) {
            if move_data.contains(&MoveFlags::InvalidRevealKing) {
                println!("Invalid Move, Watch the king");
            } else if move_data.contains(&MoveFlags::InvalidWaitingForPromotion) {
                println!("Invalid Move, Waiting for promotion");
            } else {
                println!("Invalid Move");
            }
            continue;
        }
    }
}
