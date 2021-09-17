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

    fn move_all_clone(game: Game, depth: u32) -> u32 {
        if depth == 0 {
            return 1;
        }

        let mut num_moves = 0u32;

        /*for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let pos = Position { x: x, y: y };

                let piece_data = game.board[x][y];
                if piece_data.piece != Piece::None && piece_data.is_white == game.is_white_to_move {
                    let threatmap = generate_all_threats(&game, !game.is_white_to_move);
                    let valid_moves = generate_valid_moves(&game, &threatmap, &pos);
                    for move_end in valid_moves {
                        let mut clone_board = (game).clone();
                        if move_piece_unsafe(&mut clone_board, pos, move_end).len() > 0 {
                            num_moves += move_all_clone(clone_board, depth - 1);
                        }
                    }
                }
            }
        }*/

        return num_moves;
    }

    
    fn move_all(game: &mut Game, depth: u32, time : u128) -> (u32, u128) {
        if depth == 0 {
            return (1,0);
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
                    time += now.elapsed().as_nanos();
                    
                    let valid_moves = generate_valid_moves(&game, &threatmap, &pos);
                    
                    
                    for move_end in valid_moves {
                       // let mut clone_board = game.clone();
                       
                        let castle = game.castle;
                        let half_move_clock = game.half_move_clock;
                        let full_move_clock = game.full_move_clock;
                        let is_white_to_move = game.is_white_to_move;
                        let en_passant = game.en_passant_position;
                        //let prnt1 = print_board(&game).unwrap();
                        let undo = move_piece_unsafe(game, pos, move_end);
                        
                       // let prnt3 = print_board(&game).unwrap();
                        if undo.len() > 0 {
                            
                            let result = move_all(game, depth - 1,time);
                            
                            
                            
                            num_moves += result.0;
                            time += result.1;
                            //undo move
                           // let _clone = undo.clone();
                            for (u_pos, u_data) in undo {
                                game.board[u_pos.x][u_pos.y] = u_data;
                            }
                            
                            game.castle = castle;
                            game.en_passant_position = en_passant;
                            game.is_white_to_move = is_white_to_move;
                            game.full_move_clock = full_move_clock;
                            game.half_move_clock = half_move_clock;
                            
                           /* let prnt2 = print_board(&game).unwrap();
                            if prnt1 != prnt2 {
                                println!("{}", prnt1);
                                println!("{}", prnt2);
                                println!("{}", prnt3);
                                for (u_pos, u_data) in _clone {
                                    println!(" {} : {}",print_position(u_pos),u_data.is_white);
                                    //game.board[u_pos.x][u_pos.y] = u_data;
                                }
                                println!("======");
                                return 0;
                            }*/
                            
                            //println!("{}", prnt);
                        }
                     
                    }
                    
                }
            }
        }
        
        return (num_moves,time);
    }

    /*#[test]
    fn run_deep_test() {
        let dep = 3 - 1;
        let list = vec![
            ("a2a3", 380),
("b2b3", 420),
("c2c3", 420),
("d2d3", 539),
("e2e3", 599),
("f2f3", 380),
("g2g3", 420),
("h2h3", 380),
("a2a4", 420),
("b2b4", 421),
("c2c4", 441),
("d2d4", 560),
("e2e4", 600),
("f2f4", 401),
("g2g4", 421),
("h2h4", 420),
("b1a3", 400),
("b1c3", 440),
("g1f3", 440),
("g1h3", 400),
        ];
        for q in list {
            let mut game = get_board(
                STANDARD_BOARD.to_string(),
            )
            .unwrap();
            let temp_move = parse_move(q.0).unwrap();
            move_piece_no_map(&mut game, temp_move.0, temp_move.1, true);
            //rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPPKPP/RNBQ1BNR b kq - 1 2
            //rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPRKPP/1NBQ1BNR b kq - 1 2
            let moves = move_all(&game, dep);
            if moves != q.1 {
                let board2 = print_board(&game).unwrap(); //print_board(&game).unwrap();
                println!(">>> {}", board2);
                let game2 = get_board(board2).unwrap();
                let moves2 = move_all(&game2, dep);
                println!("{} / {} at {}, check2 {}", moves, q.1, q.0, moves2);
            }
            //break;
        }
        return;
        let game = get_board(STANDARD_BOARD.to_string()).unwrap();
        assert_eq!(move_all(&game, 0), 1);
        assert_eq!(move_all(&game, 1), 20);
        assert_eq!(move_all(&game, 2), 400);
        assert_eq!(move_all(&game, 3), 8902);
        assert_eq!(move_all(&game, 4), 197281);
    }*/

    #[test]
    fn run_deep_test_undo() {
        let mut game = get_board(STANDARD_BOARD.to_string()).unwrap();
        println!("TIME {}ms", move_all(&mut game, 3,0).1 / 1_000_000);
        /*assert_eq!(move_all(&mut game, 0), 1);
        assert_eq!(move_all(&mut game, 1), 20);
        assert_eq!(move_all(&mut game, 2), 400);
        assert_eq!(move_all(&mut game, 3), 8902);
        assert_eq!(move_all(&mut game, 4), 197281);*/
    }

    #[test]
    fn run_deep_test_clone() {
        /*
        let game = get_board(STANDARD_BOARD.to_string()).unwrap();
        assert_eq!(move_all_clone(game, 0), 1);
        assert_eq!(move_all_clone(game, 1), 20);
        assert_eq!(move_all_clone(game, 2), 400);
        assert_eq!(move_all_clone(game, 3), 8902);
        assert_eq!(move_all_clone(game, 4), 197281);*/
    }
}
