
use std::collections::HashSet;

use crate::game_data::*;

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
pub(crate) fn generate_all_moves(game: &Game, piece_position: &Position) -> HashSet<Position> {
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

pub fn promote_pawn(game: &mut Game, promotion: Piece) -> bool {
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

                                // removes the has castled
                                if is_white {
                                    game.white_castle = EMPTY_CASTLE;
                                } else {
                                    game.black_castle = EMPTY_CASTLE;
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

pub fn move_piece(
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
                } else if move_start.x == BOARD_SIZE - 1 {
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
