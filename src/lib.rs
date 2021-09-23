pub mod game_data;
pub mod logic;
pub mod parser;
pub mod render;

#[cfg(test)]
mod tests {
    use crate::game_data::*;
    use crate::logic::*;
    use crate::parser::*;

    fn load_board(board: &str, moves: Vec<&str>) -> Option<(String, bool)> {
        let mut game_board = match init_game_board(board.to_string()) {
            Some(gm) => gm,
            None => return None,
        };

        let mut is_successful = true;
        for input in moves {
            let (move_start, move_end) = match parse_move(&input) {
                Some(t) => t,
                None => return None,
            };
            let has_moves = move_piece_no_map(&mut game_board, move_start, move_end, true);
            if !has_moves {
                is_successful = false;
                break;
            }
        }
        //let flags = last_flags.iter().map(|s| => s).coll;
        return Some((
            match get_fen(&game_board.game) {
                Some(str) => str,
                None => return None,
            },
            is_successful,
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
        let (board, has_moved) = load_board(str, vec!["e1e2"]).unwrap();
        assert_eq!(str, board);
        assert_eq!(false, has_moved);
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
            load_board(str, vec!["d5c6"]).unwrap().0
        );
    }

    fn total_moves(game: Game, depth: u32) -> u32 {
        return move_all(game, depth).0;
    }

    /* does all possible moves, returns as moves + time*/
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

    //https://www.chessprogramming.org/Perft_Results
    #[test]
    fn perft_test_2() {
        let game = get_board(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 0".to_string(),
        )
        .unwrap();
        assert_eq!(total_moves(game, 1), 48);
        assert_eq!(total_moves(game, 2), 2039);
        assert_eq!(total_moves(game, 3), 97862);
        // assert_eq!(total_moves(game, 4), 4085603); // 4074224 ;-;
    }

    #[test]
    fn perft_test_3() {
        let game = get_board("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 0".to_string()).unwrap();
        assert_eq!(total_moves(game, 1), 14);
        assert_eq!(total_moves(game, 2), 191);
        assert_eq!(total_moves(game, 3), 2812);
        assert_eq!(total_moves(game, 4), 43238);
    }

    #[test]
    fn perft_test_4() {
        let game = get_board(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(),
        )
        .unwrap();
        assert_eq!(total_moves(game, 1), 6);
        assert_eq!(total_moves(game, 2), 264);
        assert_eq!(total_moves(game, 3), 9467);
        // assert_eq!(total_moves(game, 4), 422333);
    }

    #[test]
    fn perft_test_5() {
        let game =
            get_board("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string())
                .unwrap();
        assert_eq!(total_moves(game, 1), 44);
        assert_eq!(total_moves(game, 2), 1486);
        assert_eq!(total_moves(game, 3), 62379);
        //  assert_eq!(total_moves(game, 4),  2103487 );
    }

    #[test]
    fn perft_test_6() {
        let game = get_board(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".to_string(),
        )
        .unwrap();

        assert_eq!(total_moves(game, 1), 46);
        assert_eq!(total_moves(game, 2), 2079);
        assert_eq!(total_moves(game, 3), 89890);
        // assert_eq!(total_moves(game, 4),  3894594 );
    }

    #[test]
    fn run_deep_test_1() {
        let game = get_board(STANDARD_BOARD.to_string()).unwrap();
        //println!("TIME {}ms", move_all(game, 3).1 / 1_000_000);
        assert_eq!(total_moves(game, 1), 20);
        assert_eq!(total_moves(game, 2), 400);
        assert_eq!(total_moves(game, 3), 8902);

        //assert_eq!(total_moves(game, 4), 197281);
        //627.90s and it works!!
        //assert_eq!(total_moves(game, 5, 0), 4865609);
    }

    #[test]
    fn run_deep_test_2() {
        let game = get_board(
            "rnbqkbnr/ppp1p2p/6p1/3p1p2/6P1/5P1N/PPPPP1BP/RNBQK2R b KQkq - 1 4".to_string(),
        )
        .unwrap();
        assert_eq!(total_moves(game, 0), 1);
        assert_eq!(total_moves(game, 1), 27);
        assert_eq!(total_moves(game, 2), 671);
        assert_eq!(total_moves(game, 3), 19218);
        //assert_eq!(total_moves(game, 4), 492460);
    }

    #[test]
    fn run_deep_test_3() {
        let game = get_board(
            "r1bqkb1r/p1pppppp/8/1p4B1/2Pn2n1/N3Q3/PP2PPPP/R3KBNR b KQkq - 1 6".to_string(),
        )
        .unwrap();
        assert_eq!(total_moves(game, 0), 1);
        assert_eq!(total_moves(game, 1), 32);
        assert_eq!(total_moves(game, 2), 1173);
        assert_eq!(total_moves(game, 3), 36787);
        //assert_eq!(total_moves(game, 4), 1373011);
    }

    #[test]
    fn run_deep_test_4() {
        let game = get_board(
            "rn2kb1r/pppq1ppp/3p3n/3Pp3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R b kq - 2 6".to_string(),
        )
        .unwrap();
        assert_eq!(total_moves(game, 0), 1);
        assert_eq!(total_moves(game, 1), 31);
        assert_eq!(total_moves(game, 2), 767);
        assert_eq!(total_moves(game, 3), 24016);
        //assert_eq!(total_moves(game, 4), 679848);
    }

    #[test]
    fn run_deep_test_5() {
        let game = get_board(
            "rn1qkbnr/p1pb1pp1/1p1p3p/4p1B1/Q1PP4/2N3P1/PP2PP1P/R3KBNR b KQkq - 0 6".to_string(),
        )
        .unwrap();
        assert_eq!(total_moves(game, 0), 1);
        assert_eq!(total_moves(game, 1), 26);
        assert_eq!(total_moves(game, 2), 1194);
        assert_eq!(total_moves(game, 3), 33003);
        //assert_eq!(total_moves(game, 4), 1419244);
    }

    #[test]
    fn temp_test_invalid_2() {
        /*running 1 test
        >>> rn2kb1r/pppq2pp/3p3n/3Ppp2/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq f6 3 7
        21592 / 21591 at f7f5 checked 26
        >>> rn2kb1r/pppq1ppp/3p4/3Ppn2/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq - 3 7
        21040 / 21039 at h6f5 checked 25
        >>> r3kb1r/pppq1ppp/2np3n/3Pp3/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq - 3 7
        24931 / 24929 at b8c6 checked 26
        test tests::temp_test_invalid_2 has been running for over 60 seconds
        >>> rn2kb1r/ppp2ppp/3p3n/3Pp3/q3P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq - 3 7
        23807 / 23805 at d7a4 checked 24
        >>> rn2kb1r/ppp2ppp/3p3n/3Ppq2/4P1b1/2N2N2/PPP1KPPP/R1BQ1B1R w kq - 3 7
        21144 / 21143 at d7f5 checked 26*/
        let list2 = vec![
            ("e2e3", 15),
            ("g2g3", 4),
            ("a5a6", 15),
            ("e2e4", 16),
            ("g2g4", 17),
            ("b4b1", 16),
            ("b4b2", 16),
            ("b4b3", 15),
            ("b4a4", 15),
            ("b4c4", 15),
            ("b4d4", 15),
            ("b4e4", 15),
            ("b4f4", 2),
            ("a5a4", 15),
        ];

        let fen2 = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let dep2 = 2;

        for q in list2 {
            let mut game = init_game_board(fen2.to_string()).unwrap();
            let temp_move = parse_move(q.0).unwrap();
            let flags = move_piece_no_map(&mut game, temp_move.0, temp_move.1, true);
            if !flags {
                println!("INVALID: {}", q.0);
            }

            let moves = move_all(game.game, dep2 - 1);
            if moves.0 != q.1 {
                let board2 = get_fen(&game.game).unwrap(); //print_board(&game).unwrap();
                println!(">>> {}", board2);
                let game2 = get_board(board2).unwrap();
                let moves2 = move_all(game2, 1);
                println!("{} / {} at {} checked {}", moves.0, q.1, q.0, moves2.0);
            }
        }
    }

    #[test]
    fn temp_test_invalid_1() {
        let list = vec![
            ("f4f3", 1),
            ("d6d5", 1),
            ("c7c6", 1),
            ("c7c5", 1),
            ("h4g5", 1),
            ("h5b5", 1),
            ("h5c5", 1),
            ("h5d5", 1),
            ("h5e5", 1),
            ("h5f5", 1),
            ("h5g5", 1),
            ("h5h6", 1),
            ("h5h7", 1),
            ("h5h8", 1),
            ("h4g3", 1),
            ("h4h3", 1),
            ("h4g4", 1),
        ];
        let fen = "8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1";

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
                        let q = get_move(pos, m_move);
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
            let mut game = init_game_board(fen.to_string()).unwrap();
            let temp_move = parse_move(q.0).unwrap();

            let flags = move_piece_no_map(&mut game, temp_move.0, temp_move.1, true);

            if !flags {
                let threatmap = generate_all_threats(&game.game, !game.game.is_white_to_move);
                let all_moves = generate_all_moves_and_castle(&game.game, &threatmap, &temp_move.0);
                let moves = generate_valid_moves(&game.game, &threatmap, &temp_move.0);
                crate::render::render_highlight(
                    &game.game,
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
