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

/**
Excludes casteling to not make an while true loop while
first is all direct attacks
second is all attacks with 1 piece blocking
third is all direct threats to the king
*/
pub(crate) fn generate_all_moves(game: &Game, piece_position: &Position) -> ThreatMap {
    let mut all_positions: HashSet<Position> = HashSet::new();
    let mut all_secondary_positions: HashSet<Position> = HashSet::new();
    let mut all_king_positions: HashSet<Position> = HashSet::new();

    let start_piece = game.board[piece_position.x][piece_position.y];
    if start_piece.piece == Piece::None {
        return ThreatMap {
            all_threats: all_positions,
            all_threats_secondary: all_secondary_positions,
            all_king_threats: all_king_positions,
        };
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
                    // aka is about to capture the king
                    if game.board[new_valid_position.x][new_valid_position.y].piece == Piece::King {
                        all_king_positions.insert(new_valid_position);
                        all_king_positions.insert(*piece_position);
                    }
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

                if !is_square_color(&game, &new_valid_position, start_piece.is_white) {
                    //aka is about to capture the king
                    if game.board[new_valid_position.x][new_valid_position.y].piece == Piece::King {
                        all_king_positions.insert(new_valid_position);
                        all_king_positions.insert(*piece_position);
                    }
                    all_positions.insert(new_valid_position);
                }
            }

            // Goes though all inf move directions
            for i_move in moveset.inf_moves {
                let mut index = 0;
                // 0 means direct attacks, 1 means attacks that jump over 1 piece
                let mut ghost_index = 0;
                let mut local_line: HashSet<Position> = HashSet::new();
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

                    local_line.insert(new_valid_position);

                    // if not the same color
                    if !is_square_color(&game, &new_valid_position, start_piece.is_white) {
                        if ghost_index == 0 {
                            all_positions.insert(new_valid_position);
                        }

                        // break if the piece is of another color, because the pawn cant ghost though pieces
                        let capture_piece = game.board[new_valid_position.x][new_valid_position.y];
                        if capture_piece.piece != Piece::None
                            && (capture_piece.is_white != start_piece.is_white)
                        {
                            // as we are only intrested in the first 2 or king, the rest can go
                            if capture_piece.piece == Piece::King {
                                if ghost_index == 0 {
                                    // if something threatens the king, then a valid move is to capture that piece
                                    all_king_positions.insert(*piece_position);
                                    for l_move in local_line {
                                        all_king_positions.insert(l_move);
                                    }
                                } else if ghost_index == 1 {
                                    for l_move in local_line {
                                        all_secondary_positions.insert(l_move);
                                    }
                                }
                                break;
                            }

                            ghost_index += 1;
                            if ghost_index >= 2 {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    ThreatMap {
        all_threats: all_positions,
        all_threats_secondary: all_secondary_positions,
        all_king_threats: all_king_positions,
    }
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
    if is_square_color(&game, move_end, game.is_white_to_move) {
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
pub fn generate_all_threats(game: &Game, is_white: bool) -> ThreatMap {
    let mut all_threats: HashSet<Position> = HashSet::new();
    let mut all_threats_secondary: HashSet<Position> = HashSet::new();
    let mut all_king_threats: HashSet<Position> = HashSet::new();
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let position = Position { x, y };
            if is_square_color(game, &position, is_white) {
                let piece_threads = generate_all_moves(game, &position);
                for t_pos in piece_threads.all_threats {
                    all_threats.insert(t_pos);
                }
                for t_pos in piece_threads.all_threats_secondary {
                    all_threats_secondary.insert(t_pos);
                }
                for t_pos in piece_threads.all_king_threats {
                    all_king_threats.insert(t_pos);
                }
            }
        }
    }

    ThreatMap {
        all_threats: all_threats,
        all_threats_secondary: all_threats_secondary,
        all_king_threats: all_king_threats,
    }
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

fn get_castle_positions(
    game: &Game,
    piece_position: &Position,
    is_white: bool,
) -> HashSet<Position> {
    let mut moves = HashSet::new();

    let castle_status = if is_white {
        &game.castle[0]
    } else {
        &game.castle[1]
    };

    let piece_data = game.board[piece_position.x][piece_position.y];

    // has no king... how? or cant castle
    if piece_data.is_white != is_white
        || piece_data.piece != Piece::King
        || (!castle_status.can_castle_king_side && !castle_status.can_castle_queen_side)
    {
        return moves;
    }

    // be aware that you can trick this by adding a second rook because
    // it only keeps track of the first rook on the left or right side
    let spawn_y = if is_white { WHITE_SPAWN } else { BLACK_SPAWN };

    // right y pos
    if spawn_y == piece_position.y {
        let other_threats = generate_all_threats(game, !is_white);

        // king cant castle if checked
        const KING_SIDES: [i8; 2] = [-1, 1];
        for offset in KING_SIDES {
            if !other_threats.all_king_threats.contains(piece_position) {
                let is_king_side = offset > 0;

                // if can castle
                if if is_king_side {
                    castle_status.can_castle_king_side
                } else {
                    castle_status.can_castle_queen_side
                } {
                    let rook_pos = if is_king_side {
                        castle_status.king_side_rook
                    } else {
                        castle_status.queen_side_rook
                    };

                    for index in 1..BOARD_SIZE {
                        let new_valid_position = match get_position(
                            piece_position,
                            &Vector2 {
                                x: (index as i8) * offset,
                                y: 0,
                            },
                        ) {
                            Some(pos) => pos,
                            None => break,
                        };

                        // the king cant be checked on his way over to the rook,
                        // because the king only moves 2 squares, the index is 2
                        if other_threats.all_king_threats.contains(&new_valid_position)
                            && index <= 2
                        {
                            break;
                        }

                        let piece_data = game.board[new_valid_position.x][new_valid_position.y];

                        // found rook
                        if piece_data.is_white == is_white
                            && piece_data.piece == Piece::Rook
                            && new_valid_position == rook_pos
                        {
                            let new_valid_king = match get_position(
                                piece_position,
                                &Vector2 {
                                    x: 2i8 * offset,
                                    y: 0,
                                },
                            ) {
                                Some(pos) => pos,
                                None => break,
                            };
                            moves.insert(new_valid_king);
                        } else if piece_data.piece != Piece::None {
                            // cant jump over pieces
                            break;
                        }
                    }
                }
            }
        }
    }
    return moves;
}

pub fn generate_all_moves_and_castle(game: &Game, piece_position: &Position) -> HashSet<Position> {
    if piece_position.x >= BOARD_SIZE || piece_position.y >= BOARD_SIZE {
        return HashSet::new();
    }
    let mut moves = generate_all_moves(&game, piece_position).all_threats;

    let start_piece = game.board[piece_position.x][piece_position.y];
    if start_piece.piece == Piece::King {
        let castle_moves = get_castle_positions(game, piece_position, start_piece.is_white);
        for c_move in castle_moves {
            moves.insert(c_move);
        }
    }

    return moves;
}

pub fn generate_valid_moves_for_team(
    game: &Game,
    other_team_threat_map: &ThreatMap,
    is_white: bool,
) -> HashSet<Position> {
    let mut map: HashSet<Position> = HashSet::new();

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let pos = &Position { x: x, y: y };
            if is_square_color(game, pos, is_white) {
                for v_pos in generate_valid_moves(game, other_team_threat_map, pos) {
                    map.insert(v_pos);
                }
            }
        }
    }
    return map;
}

pub fn generate_valid_moves(
    game: &Game,
    other_team_threat_map: &ThreatMap,
    piece_position: &Position,
) -> HashSet<Position> {
    let mut valid_positions: HashSet<Position> = HashSet::new();

    let all_positions = generate_all_moves(game, piece_position);
    if all_positions.all_threats.len() == 0 {
        return valid_positions;
    }

    let piece_data = game.board[piece_position.x][piece_position.y];
    let is_white = piece_data.is_white;

    // 1. find king threats
    if other_team_threat_map.all_king_threats.len() > 0 {
        if piece_data.piece == Piece::King {
            for pos in all_positions.all_threats {
                if !is_square_color(game, &pos, is_white)
                    && !other_team_threat_map.all_threats.contains(&pos)
                {
                    valid_positions.insert(pos);
                }
            }
        } else {
            // place infront of king or capture
            // but cant move if it will reveal the king
            if !other_team_threat_map
                .all_threats_secondary
                .contains(piece_position)
            {
                for pos in all_positions.all_threats {
                    if !is_square_color(game, &pos, is_white)
                        && other_team_threat_map.all_king_threats.contains(&pos)
                    {
                        valid_positions.insert(pos);
                    }
                }
            }
        }

        return valid_positions;
    }

    // 2. check if is secondary
    let will_reveal_king = other_team_threat_map
        .all_threats_secondary
        .contains(piece_position)
        && piece_data.piece != Piece::King;

    // be aware that this will fail for a piece that is moving from 1 reveal to another
    for pos in all_positions.all_threats {
        if !is_square_color(game, &pos, is_white)
            && (!will_reveal_king || other_team_threat_map.all_threats_secondary.contains(&pos))
        {
            valid_positions.insert(pos);
        }
    }

    if piece_data.piece == Piece::King {
        let castle = get_castle_positions(game, piece_position, is_white);
        for c_pos in castle {
            valid_positions.insert(c_pos);
        }
    }

    return valid_positions;
}

pub fn move_piece(
    game: &mut Game,
    move_start: &Position,
    move_end: &Position,
    auto_promote: bool,
) -> HashSet<MoveFlags> {
    let mut flags: HashSet<MoveFlags> = HashSet::new();
    flags.insert(MoveFlags::Invalid);
    return flags;

    /*
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
    flags*/
}
