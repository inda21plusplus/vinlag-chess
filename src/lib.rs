pub mod game_data;
pub mod logic;
pub mod parser;
pub mod render;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::game_data::*;
    use crate::logic::*;
    use crate::parser::*;

    fn load_board(board: &str, moves: Vec<&str>) -> Option<(String, HashSet<MoveFlags>)> {
        let mut game = match get_board(board.to_string()) {
            Some(gm) => gm,
            None => return None,
        };

        let mut last_flags = HashSet::new();
        last_flags.insert(MoveFlags::Valid);

        for input in moves {
            let (move_start, move_end) = match parse_move(&input) {
                Some(t) => t,
                None => return None,
            };
            let flags = move_piece_no_map(&mut game, move_start, move_end, true);
            last_flags = flags;
            if last_flags.contains(&MoveFlags::Invalid) {
                break;
            }
        }
        //let flags = last_flags.iter().map(|s| => s).coll;
        return Some((
            match print_board(&game) {
                Some(str) => str,
                None => return None,
            },
            last_flags,
        ));
    }

    //https://www.chess.com/analysis

    #[test]
    fn simple_move() {
        let str = STANDARD_BOARD;
        assert_eq!(
            "rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2",
            load_board(str, vec!["e2e4", "f7f5", "e4f5"]).unwrap().0
        );
    }

    #[test]
    fn simple_invalid() {
        let str = STANDARD_BOARD;
        let board = load_board(str, vec!["e1e2"]).unwrap();
        assert_eq!(str, board.0);
        assert_eq!(true, board.1.contains(&MoveFlags::Invalid));
    }

    #[test]
    fn clock_test() {
        let str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 4 3",
            load_board(str, vec!["b1c3", "b8c6", "c3b1", "c6b8"])
                .unwrap()
                .0
        );
    }

    #[test]
    fn en_passant_test() {
        let str = "rn2kb1r/pp1q1ppp/3p3n/2pPp3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq c6 0 7";
        assert_eq!(
            "rn2kb1r/pp1q1ppp/2Pp3n/4p3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R b kq - 0 7",
            load_board(str, vec!["d5c6"])
                .unwrap()
                .0
        );
    }

    fn move_all(game: Game, depth: u32) -> (u32, u128) {
        if depth == 0 {
            return (1, 0);
        }

        let mut num_moves = 0u32;
        let mut time = 0;
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let pos = Position { x: x, y: y };

                let piece_data = game.board[x][y];
                if piece_data.piece != Piece::None && piece_data.is_white == game.is_white_to_move {
                    let now = std::time::Instant::now();
                    let threatmap = generate_all_threats(&game, !game.is_white_to_move);

                    let valid_moves = generate_valid_moves(&game, &threatmap, &pos);
                    time += now.elapsed().as_nanos();
                    for move_end in valid_moves {
                        let mut clone_board = game.clone();
                        if move_piece_unsafe(&mut clone_board, pos, move_end) {
                            let (r_moves, r_time) = move_all(clone_board, depth - 1);
                            num_moves += r_moves;
                            time += r_time;
                        }
                    }
                }
            }
        }
        return (num_moves, time);
    }

    #[test]
    fn run_deep_test_1() {
        let game = get_board(STANDARD_BOARD.to_string()).unwrap();
        //println!("TIME {}ms", move_all(game, 3).1 / 1_000_000);
        assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 20);
        assert_eq!(move_all(game, 2).0, 400);
        assert_eq!(move_all(game, 3).0, 8902);

        //assert_eq!(move_all(game, 4).0, 197281);
        //627.90s and it works!!
        //assert_eq!(move_all(game, 5, 0).0, 4865609);
    }

    #[test]
    fn run_deep_test_2() {
        let game = get_board(
            "rnbqkbnr/ppp1p2p/6p1/3p1p2/6P1/5P1N/PPPPP1BP/RNBQK2R b KQkq - 1 4".to_string(),
        )
        .unwrap();
        assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 27);
        assert_eq!(move_all(game, 2).0, 671);
        assert_eq!(move_all(game, 3).0, 19218);
        //assert_eq!(move_all(game, 4).0, 492460);
    }

    #[test]
    fn run_deep_test_3() {
        let game = get_board(
            "r1bqkb1r/p1pppppp/8/1p4B1/2Pn2n1/N3Q3/PP2PPPP/R3KBNR b KQkq - 1 6".to_string(),
        )
        .unwrap();
        assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 32);
        assert_eq!(move_all(game, 2).0, 1173);
        assert_eq!(move_all(game, 3).0, 36787);
        assert_eq!(move_all(game, 4).0, 1373011); // check 1373021 vs 1373011
    }

    #[test]
    fn run_deep_test_4() {
        let game = get_board(
            "rn2kb1r/pppq1ppp/3p3n/3Pp3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R b kq - 2 6".to_string(),
        )
        .unwrap();
        assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 31);
        assert_eq!(move_all(game, 2).0, 767);
        assert_eq!(move_all(game, 3).0, 24016);
        assert_eq!(move_all(game, 4).0, 679848); // check 679856 vs 679848
    }

    #[test]
    fn run_deep_test_5() {
        let game = get_board(
            "rn1qkbnr/p1pb1pp1/1p1p3p/4p1B1/Q1PP4/2N3P1/PP2PP1P/R3KBNR b KQkq - 0 6".to_string(),
        )
        .unwrap();
        assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 26);
        assert_eq!(move_all(game, 2).0, 1194);
        assert_eq!(move_all(game, 3).0, 33003);
        //assert_eq!(move_all(game, 4).0, 1419244);
    }

    #[test]
    fn temp_test_invalid_2() {
        let list2 = vec![
            ("a2a3", 31),
            ("b2b3", 31),
            ("g2g3", 31),
            ("h2h3", 31),
            ("a2a4", 31),
            ("b2b4", 32),
            ("h2h4", 31),
            ("d5c6", 30),
            ("c3b1", 31),
            ("c3a4", 31),
            ("c3b5", 29),
            ("e2e3", 31),
            ("e2d3", 31),
            ("e2d2", 31),
            ("e2e1", 31),
            ("d1d4", 33),
            ("d1d3", 31),
            ("d1d2", 31),
            ("c1d2", 31),
            ("c1e3", 31),
            ("c1f4", 32),
            ("c1g5", 28),
            ("c1h6", 30),
            ("a1b1", 31),
            ("h1g1", 31),
            ("d1e1", 31),
        ];
        let fen2 = "rn2kb1r/pp1q1ppp/3p3n/2pPp3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq c6 0 7";
        let dep2 = 2;

        for q in list2 {
            let mut game = get_board(fen2.to_string()).unwrap();
            let temp_move = parse_move(q.0).unwrap();
            let flags = move_piece_no_map(&mut game, temp_move.0, temp_move.1, true);
            if flags.contains(&MoveFlags::Invalid) {
                println!("INVALID: {}", q.0);
            }

            let moves = move_all(game, dep2 - 1);
            if moves.0 != q.1 {
                let board2 = print_board(&game).unwrap(); //print_board(&game).unwrap();
                 println!(">>> {}", board2);
                 let game2 = get_board(board2).unwrap();
                 let moves2 = move_all(game2, 1);
                println!("{} / {} at {} checked {}", moves.0, q.1, q.0,moves2.0);
            }
        }
    }

    #[test]
    fn temp_test_invalid_1() {
        let list = vec![
            ("d6d5", 1),
("a7a6", 1),
("b7b6", 1),
("f7f6", 1),
("g7g6", 1),
("a7a5", 1),
("b7b5", 1),
("f7f5", 1),
("g7g5", 1),
("b7c6", 1),
("h6f5", 1),
("h6g8", 1),
("b8a6", 1),
("b8c6", 1),
("g4f3", 1),
("g4h3", 1),
("g4f5", 1),
("g4h5", 1),
("g4e6", 1),
("f8e7", 1),
("h8g8", 1),
("d7f5", 1),
("d7c6", 1),
("d7e6", 1),
("d7c7", 1),
("d7e7", 1),
("d7c8", 1),
("d7d8", 1),
("e8e7", 1),
("e8d8", 1),
        ];
        let fen = "rn2kb1r/pp1q1ppp/2Pp3n/4p3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R b kq - 0 7";

        let game2 = get_board(fen.to_string()).unwrap();
        let threatmap2 = generate_all_threats(&game2, !game2.is_white_to_move);

        let mut size = 0;
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let pos = Position { x: x, y: y };
                if is_square_color(&game2, &pos, game2.is_white_to_move) {
                    let moves = generate_valid_moves(&game2, &threatmap2, &pos);
                    for m_move in moves {
                        size += 1;
                        let q = print_move(pos, m_move);
                        let mut contains_q = false;
                        for z in list.clone() {
                            if z.0 == q {
                                contains_q = true;
                            }
                        }
                        if !contains_q {
                            let threatmap = generate_all_threats(&game2, !game2.is_white_to_move);
                            let all_moves = generate_valid_moves(&game2, &threatmap, &pos);
                            let t_moves = generate_valid_moves(&game2, &threatmap, &pos);
                            crate::render::render_highlight(
                                &game2,
                                vec![
                                    (&threatmap.all_threats, termcolor::Color::Red),
                                    // (&threatmap.all_threats, termcolor::Color::Red),
                                ],
                            );
                            println!("ILEGAL MOVE: {}", q);
                        }
                    }
                }
            }
        }

        //if size != list.len() {
            println!("QQ MOVE: {}", size);
        //}

        for q in list.clone() {
            let mut game = get_board(fen.to_string()).unwrap();
            let temp_move = parse_move(q.0).unwrap();

            let flags = move_piece_no_map(&mut game, temp_move.0, temp_move.1, true);

            if flags.contains(&MoveFlags::Invalid) {
                let threatmap = generate_all_threats(&game, !game.is_white_to_move);
                let all_moves = generate_all_moves_and_castle(&game, &threatmap, &temp_move.0);
                let moves = generate_valid_moves(&game, &threatmap, &temp_move.0);
                crate::render::render_highlight(
                    &game,
                    vec![
                        //(&all_moves, termcolor::Color::Blue),
                        (&threatmap.all_king_threats_full, termcolor::Color::Red),
                    ],
                );
                println!("Did not catch move {}", q.0);
                break;
            }
        }
    }
}
