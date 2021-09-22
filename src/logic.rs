use std::collections::HashSet;

use crate::{game_data::*, parser::get_board_fen};

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

pub(crate) fn is_square_color(game: &Game, piece_position: &Position, is_white: bool) -> bool {
    let piece_data = game.board[piece_position.x][piece_position.y];
    return piece_data.piece != Piece::None && piece_data.is_white == is_white;
}

/**
Excludes casteling to not make an while true loop
*/
pub(crate) fn generate_all_moves(game: &Game, piece_position: &Position) -> ThreatMap {
    let mut all_moves: HashSet<Position> = HashSet::new();
    let mut all_threats: HashSet<Position> = HashSet::new();
    let mut all_pinned: Vec<HashSet<Position>> = Vec::new();
    let mut all_king_threats: HashSet<Position> = HashSet::new();
    let mut all_king_threats_full: HashSet<Position> = HashSet::new();

    let start_piece = game.board[piece_position.x][piece_position.y];
    if start_piece.piece == Piece::None {
        return ThreatMap {
            all_moves,
            all_threats,
            all_pinned,
            all_king_threats,
            all_king_threats_full,
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
                all_moves.insert(pos_advance);

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
                        all_moves.insert(pos);
                    }
                };
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
                    all_moves.insert(new_valid_position);
                } else if !is_square_color(game, &new_valid_position, start_piece.is_white) {
                    // I dont want the pawn to be able to move sideways, but to threaten the sideways pieces

                    // aka is about to capture the king
                    if game.board[new_valid_position.x][new_valid_position.y].piece == Piece::King {
                        all_king_threats.insert(new_valid_position);
                        all_king_threats.insert(*piece_position);
                    }
                    if is_square_color(game, &new_valid_position, !start_piece.is_white) {
                        all_moves.insert(new_valid_position);
                    }
                }
                all_threats.insert(new_valid_position);
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
                        all_king_threats.insert(new_valid_position);
                        all_king_threats.insert(*piece_position);
                    }
                    all_moves.insert(new_valid_position);
                }
                all_threats.insert(new_valid_position);
            }

            // Goes though all inf move directions
            for i_move in moveset.inf_moves {
                let mut index = 0;
                // 0 means direct attacks, 1 means attacks that jump over 1 piece
                let mut ghost_index = 0;
                let mut local_line: HashSet<Position> = HashSet::new();
                local_line.insert(*piece_position);
                //let mut is_king = -1;
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
                            all_moves.insert(new_valid_position);
                            all_threats.insert(new_valid_position);
                        }

                        // break if the piece is of another color, because the pawn cant ghost though pieces
                        let capture_piece = game.board[new_valid_position.x][new_valid_position.y];
                        if capture_piece.piece != Piece::None
                        //&& (capture_piece.is_white != start_piece.is_white)
                        {
                            // as we are only intrested in the first 2 or king, the rest can go
                            if capture_piece.piece == Piece::King {
                                if ghost_index == 0 {
                                    // if something threatens the king, then a valid move is to capture that piece
                                    all_king_threats.insert(*piece_position);
                                    for l_move in local_line {
                                        all_king_threats.insert(l_move);
                                    }

                                    all_king_threats_full = all_king_threats.clone();

                                    let new_extra_move = Vector2 {
                                        x: i_move.x * (index + 1),
                                        y: i_move.y * (index + 1),
                                    };
                                    let new_extra =
                                        match get_position(piece_position, &new_extra_move) {
                                            Some(pos) => pos,
                                            None => break,
                                        };
                                    all_king_threats_full.insert(new_extra);
                                } else if ghost_index == 1 {
                                    all_pinned.push(local_line);
                                }
                                break;
                            }

                            ghost_index += 1;
                            if ghost_index >= 2 {
                                break;
                            }
                        }
                    } else {
                        if ghost_index == 0 {
                            all_threats.insert(new_valid_position);
                        }
                        break;
                    }
                }
                /*if is_king == 0 {
                    // if something threatens the king, then a valid move is to capture that piece
                    all_king_positions.insert(*piece_position);
                    for l_move in local_line {
                        all_king_positions.insert(l_move);
                    }
                } else if ghost_index == 1 {
                    all_secondary_positions.push(local_line);
                }*/
            }
        }
    }

    ThreatMap {
        all_moves,
        all_threats,
        all_pinned,
        all_king_threats,
        all_king_threats_full,
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

/** returns true if the promotion took place */
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

pub fn get_threats(game_board: &Gameboard)-> ThreatMap {
    return generate_all_threats(&game_board.game,!game_board.game.is_white_to_move);
}

/** Includes all threat positions generate by that team */
pub(crate) fn generate_all_threats(game: &Game, is_white: bool) -> ThreatMap {
    let mut all_moves: HashSet<Position> = HashSet::new();
    let mut all_threats: HashSet<Position> = HashSet::new();
    let mut all_pinned: Vec<HashSet<Position>> = Vec::new();
    let mut all_king_threats: HashSet<Position> = HashSet::new();
    let mut all_king_threats_full: HashSet<Position> = HashSet::new();

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let position = Position { x, y };
            if is_square_color(game, &position, is_white) {
                let piece_threads = generate_all_moves(game, &position);
                for t_pos in piece_threads.all_pinned {
                    all_pinned.push(t_pos);
                }
                for t_pos in piece_threads.all_threats {
                    all_threats.insert(t_pos);
                }
                for t_pos in piece_threads.all_king_threats {
                    all_king_threats.insert(t_pos);
                }
                for t_pos in piece_threads.all_king_threats_full {
                    all_king_threats_full.insert(t_pos);
                }
                for t_pos in piece_threads.all_moves {
                    all_moves.insert(t_pos);
                }
            }
        }
    }

    ThreatMap {
        all_moves,
        all_threats,
        all_pinned,
        all_king_threats,
        all_king_threats_full,
    }
}

/*fn find_king(game: &Game, is_white: bool) -> Option<Position> {
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let piece_data = game.board[x][y];
            if piece_data.piece == Piece::King && piece_data.is_white == is_white {
                return Some(Position { x, y });
            }
        }
    }
    None
}*/

fn get_castle_positions(
    game: &Game,
    other_team_threat_map: &ThreatMap,
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
        // king cant castle if checked
        const KING_SIDES: [i8; 2] = [-1, 1];
        for offset in KING_SIDES {
            if !other_team_threat_map
                .all_king_threats
                .contains(piece_position)
            {
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
                        if other_team_threat_map
                            .all_threats
                            .contains(&new_valid_position)
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

pub(crate) fn generate_all_moves_and_castle(
    game: &Game,
    other_team_threat_map: &ThreatMap,
    piece_position: &Position,
) -> HashSet<Position> {
    if piece_position.x >= BOARD_SIZE || piece_position.y >= BOARD_SIZE {
        return HashSet::new();
    }
    let mut moves = generate_all_moves(&game, piece_position).all_moves;

    let start_piece = game.board[piece_position.x][piece_position.y];
    if start_piece.piece == Piece::King {
        let castle_moves = get_castle_positions(
            game,
            other_team_threat_map,
            piece_position,
            start_piece.is_white,
        );
        for c_move in castle_moves {
            moves.insert(c_move);
        }
    }

    return moves;
}

/**Used for debugging */
pub(crate) fn generate_valid_moves_for_team(
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

pub fn get_all_valid_moves(game_board: &Gameboard,
    other_team_threat_map: &ThreatMap,
    piece_position: &Position,
) -> HashSet<Position> {
    return generate_valid_moves(&game_board.game,other_team_threat_map,piece_position);
}

/** TODO en passant fuckery */
pub(crate) fn generate_valid_moves(
    game: &Game,
    other_team_threat_map: &ThreatMap,
    piece_position: &Position,
) -> HashSet<Position> {
    let mut valid_positions: HashSet<Position> = HashSet::new();

    let all_positions = generate_all_moves_and_castle(game, other_team_threat_map, piece_position);
    if all_positions.len() == 0 {
        return valid_positions;
    }

    let piece_data = game.board[piece_position.x][piece_position.y];
    let is_white = piece_data.is_white;

    // 1. find king threats
    if other_team_threat_map.all_king_threats.len() > 0 {
        if piece_data.piece == Piece::King {
            for pos in all_positions {
                // 1. King cant capture own color
                // 2. King can capture the one that threatens the king, but cant capture a piece that threats it if it is also threatend
                if !is_square_color(game, &pos, is_white) {
                    if (is_square_color(game, &pos, !is_white)
                        && !other_team_threat_map.all_threats.contains(&pos))
                        || (!other_team_threat_map.all_threats.contains(&pos)
                            && !other_team_threat_map.all_king_threats_full.contains(&pos))
                    {
                        valid_positions.insert(pos);
                    }
                }
            }
        } else {
            let mut is_secondary = false;
            for local_list in &other_team_threat_map.all_pinned {
                if local_list.contains(piece_position) {
                    is_secondary = true;
                    break;
                }
            }

            // place infront of king or capture
            // but cant move if it will reveal the king
            if !is_secondary {
                for pos in all_positions {
                    if !is_square_color(game, &pos, is_white)
                        && other_team_threat_map.all_king_threats.contains(&pos)
                    {
                        //println!("{}", print_position(pos));

                        valid_positions.insert(pos);
                    }
                }
            }
        }

        return valid_positions;
    }

    // 2. check if is secondary else normal
    if piece_data.piece != Piece::King {
        for local_list in &other_team_threat_map.all_pinned {
            if local_list.contains(piece_position) {
                for pos in all_positions {
                    if local_list.contains(&pos) {
                        valid_positions.insert(pos);
                    }
                }
                return valid_positions;
            }
        }
        for pos in all_positions {
            valid_positions.insert(pos);
        }
    } else {
        for pos in all_positions {
            if !other_team_threat_map.all_threats.contains(&pos) {
                // you cant capture a pinned piece with your king
                let mut cant_capture = false;
                if is_square_color(game, &pos, !is_white) {
                    for local_list in &other_team_threat_map.all_pinned {
                        if local_list.contains(&pos) {
                            cant_capture = true;
                            break;
                        }
                    }
                }
                if cant_capture {
                    continue;
                }

                valid_positions.insert(pos);
            }
        }
        return valid_positions;
    }
    return valid_positions;
}

fn remove_castle(game: &mut Game, pos: Position) {
    let start_piece = game.board[pos.x][pos.y];

    let is_white = start_piece.is_white;

    let player_index = if is_white { 0 } else { 1 };
    let mut castle_status = game.castle[player_index];
    if castle_status.queen_side_rook == pos {
        castle_status.can_castle_queen_side = false;
    } else if castle_status.king_side_rook == pos {
        castle_status.can_castle_king_side = false;
    }
    game.castle[player_index] = castle_status;
}

/** Will go horrible wrong if input is not checked, only use if you have checked the move beforehand with valid moves
Will return true if move is done
*/
pub(crate) fn move_piece_unsafe(game: &mut Game, move_start: Position, move_end: Position) -> bool {
    let start_piece = game.board[move_start.x][move_start.y];

    if start_piece.piece == Piece::None {
        return false;
    }

    let is_white = start_piece.is_white;
    let mut half_move_clock = game.half_move_clock + 1;
    let mut en_passant_position: Option<Position> = None;

    let capture_piece = game.board[move_end.x][move_end.y];
    // reset on any capture
    if capture_piece.piece != Piece::None || start_piece.piece == Piece::Pawn {
        half_move_clock = 0;
    }

    if start_piece.piece != Piece::King && start_piece.piece != Piece::Pawn {
        game.board[move_end.x][move_end.y] = start_piece;
        game.board[move_start.x][move_start.y] = EMPTY_PEICE;
    } else if start_piece.piece == Piece::Pawn {
        let move_direction: i8 = if is_white { -1 } else { 1 };

        // en passant
        if game.board[move_end.x][move_end.y].piece == Piece::None && (move_end.x != move_start.x) {
            let real_capture_unchecked = get_position(
                &move_end,
                &Vector2 {
                    x: 0,
                    y: -move_direction,
                },
            );

            // captures the real pawn and not just air
            if real_capture_unchecked.is_some() {
                let real_capture = real_capture_unchecked.unwrap();
                let real_capture_piece_data = game.board[real_capture.x][real_capture.y];
                if real_capture_piece_data.is_white != is_white
                    && real_capture_piece_data.piece == Piece::Pawn
                {
                    game.board[real_capture.x][real_capture.y] = EMPTY_PEICE;
                }
            }
        } else if move_end.y as i8 - move_start.y as i8 == move_direction * 2 {
            // if move twice, it means that it can be en passanted
            en_passant_position = get_position(
                &move_start,
                &Vector2 {
                    x: 0,
                    y: move_direction,
                },
            )
        }

        game.board[move_end.x][move_end.y] = start_piece;
        game.board[move_start.x][move_start.y] = EMPTY_PEICE;
    } else if start_piece.piece == Piece::King {
        let offset_x = move_end.x as i8 - move_start.x as i8;
        let move_distance = i8::abs(offset_x); //i8::abs(move_start.y as i8 - move_end.y as i8) + i8::abs(offset_x);

        let player_index = if is_white { 0 } else { 1 };
        let mut castle_state = game.castle[player_index];
        castle_state.can_castle_king_side = false;
        castle_state.can_castle_queen_side = false;
        game.castle[player_index] = castle_state;

        if move_distance == 1 || move_distance == 0 {
            // if only move 1 square then it is a normal move
            game.board[move_end.x][move_end.y] = start_piece;
            game.board[move_start.x][move_start.y] = EMPTY_PEICE;
        } else if move_distance == 2 {
            // if moves 2 squares it means that it is doing castling
            // rook position = king end move -movedirection x
            let new_rook_position = match get_position(
                &move_end,
                &Vector2 {
                    x: -i8::signum(offset_x),
                    y: 0,
                },
            ) {
                Some(pos) => pos,
                None => return false,
            };

            let is_king_side = offset_x > 1;

            let caste = &game.castle[if is_white { 0 } else { 1 }];

            let rook_position = if is_king_side {
                caste.king_side_rook
            } else {
                caste.queen_side_rook
            };

            let rook_data = game.board[rook_position.x][rook_position.y];
            // moves rook
            game.board[new_rook_position.x][new_rook_position.y] = rook_data;

            // moves king
            game.board[move_end.x][move_end.y] = game.board[move_start.x][move_start.y];

            // clears old
            game.board[rook_position.x][rook_position.y] = EMPTY_PEICE;
            game.board[move_start.x][move_start.y] = EMPTY_PEICE;
        }
    }

    let full_move_clock = if is_white {
        game.full_move_clock
    } else {
        game.full_move_clock + 1
    };

    if start_piece.piece == Piece::Rook {
        remove_castle(game, move_start);
    }

    if capture_piece.piece == Piece::Rook {
        remove_castle(game, move_end);
    }

    game.half_move_clock = half_move_clock;
    game.full_move_clock = full_move_clock;
    game.is_white_to_move = !is_white;
    game.en_passant_position = en_passant_position;
    return true;
}

pub fn get_game_state(
    game_board: &Gameboard,
    other_team_threat_map: &ThreatMap,
    force_3_fold_tie: bool,
) -> WinStatus {
    let mut can_move_anything = false;

    for y in 0..BOARD_SIZE {
        // fuck goto
        if can_move_anything {
            break;
        }
        for x in 0..BOARD_SIZE {
            let piece_data = game_board.game.board[x][y];
            if piece_data.is_white == game_board.game.is_white_to_move {
                let moves = generate_valid_moves(
                    &game_board.game,
                    other_team_threat_map,
                    &Position { x, y },
                );
                if moves.len() > 0 {
                    can_move_anything = true;
                    break;
                }
            }
        }
    }

    if !can_move_anything {
        // if cant move anything and king is threatend
        if other_team_threat_map.all_king_threats.len() > 0 {
            return if game_board.game.is_white_to_move {
                WinStatus::BlackWon
            } else {
                WinStatus::WhiteWon
            };
        } else {
            // if cant move anything and king is not threatend
            return WinStatus::Tie;
        }
    }

    // If the halfmove clock becomes greater or equal than 100,
    // and the side to move has at least one legal move, a draw score should be assigned to that node,
    // with appropriate protocol handling and game state transitions, if the node is already the root and there is no mate in one.
    // https://www.chessprogramming.org/Fifty-move_Rule
    if game_board.game.half_move_clock >= 100 {
        return WinStatus::Tie;
    }

    let mut max_repetitions = 0;
    for repetitions in game_board.same_board.values() {
        if *repetitions > max_repetitions {
            max_repetitions = *repetitions;
        }
    }

    // In chess, the threefold repetition rule states that a player may claim a draw if the same position occurs three times
    // By contrast, the fivefold repetition rule requires the arbiter to
    // intervene and declare the game drawn if the same position occurs five times, and requires no claim by the players.
    // https://en.wikipedia.org/wiki/Threefold_repetition
    if (max_repetitions >= 3 && force_3_fold_tie) || (max_repetitions >= 5) {
        return WinStatus::Tie;
    }

    return WinStatus::Nothing;
}

/**Used for debugging */
pub(crate) fn move_piece_no_map(
    game_board: &mut Gameboard,
    move_start: Position,
    move_end: Position,
    auto_promote: bool,
) -> bool {
    let threatmap = generate_all_threats(&game_board.game, !game_board.game.is_white_to_move);
    return move_piece(game_board, move_start, move_end, &threatmap, auto_promote);
}

pub fn move_piece(
    game_board: &mut Gameboard,
    move_start: Position,
    move_end: Position,
    other_team_threat_map: &ThreatMap,
    auto_promote: bool,
) -> bool {
    let game = &mut game_board.game;

    if get_promotion_pawn(game).is_some() {
        return false;
    }

    // basic check first
    if !is_valid_move(game, &move_start, &move_end) {
        return false;
    }

    if !generate_valid_moves(game, other_team_threat_map, &move_start).contains(&move_end) {
        return false;
    }

    let start_piece = game.board[move_start.x][move_start.y];

    if move_piece_unsafe(game, move_start, move_end) {
        if start_piece.piece == Piece::Pawn && auto_promote {
            promote_pawn(game, Piece::Queen);
        }

        let fen = get_board_fen(game);
        //this should always work
        if fen.is_some() {
            //adds 1 if found, else sets it to 1
            *game_board.same_board.entry(fen.unwrap()).or_insert(0) += 1u8;
        }

        return true;
    } else {
        return false;
    }
}
