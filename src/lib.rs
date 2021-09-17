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
            let flags = move_piece_no_map(&mut game, &move_start, &move_end, true);
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

    fn move_all(game: Game, depth: u32) -> u32 {
        if depth == 0 {
            return 1;
        }

        let mut num_moves = 0u32;

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let pos = Position { x: x, y: y };

                let piece_data = game.board[x][y];
                if piece_data.piece != Piece::None && piece_data.is_white == game.is_white_to_move {
                    let threatmap = generate_all_threats(&game, !game.is_white_to_move);
                    let valid_moves = generate_valid_moves(&game, &threatmap, &pos);
                    for move_end in valid_moves {
                        let mut clone_board = (game).clone();
                        if move_piece_unsafe(&mut clone_board, &pos, &move_end) {
                            num_moves += move_all(clone_board, depth - 1);
                        }
                    }
                }
            }
        }

        return num_moves;
    }

    #[test]
    fn run_deep_test() {
        /*
        //rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 2
        //rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 2
        let mut temp_game =
            get_board("rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 2".to_string())
                .unwrap();

        let mut temp_move = parse_move("e1f2").unwrap();
        move_piece_unsafe(&mut temp_game, &temp_move.0, &temp_move.1);
        let d1 = move_all(temp_game, 1);

        

        let temp_game_2 =
            get_board("rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPPKPP/RNBQ1BNR b kq - 1 2".to_string())
                .unwrap();
        let d2 = move_all(temp_game_2, 1);

        println!("{}",print_board(&temp_game).unwrap());
        println!("{}",print_board(&temp_game_2).unwrap());

        println!("{} {} / 31", d1, d2);
        let mut q2: Vec<String> = Vec::new();

        let temp_game =
            get_board("rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPRKPP/1NBQ1BNR b kq - 1 2".to_string())
                .unwrap();
        let f1 = move_all(temp_game, 1);
        let threats = generate_all_threats(&temp_game, !temp_game.is_white_to_move);
        let mut size = 0;
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let pos = Position { x: x, y: y };
                let piece = temp_game.board[x][y];
                if piece.is_white == temp_game.is_white_to_move && piece.piece != Piece::None {
                    let moves = generate_valid_moves(&temp_game, &threats, &pos);
                    for c_move in moves {
                        let str = print_move(pos, c_move);

                       // println!("{}", str);
                        q2.push(str);
                        size += 1;
                    }
                }
            }
        }

        println!("sizecheck {} | {} / 31", size, f1);
       // return;
        //return;
        let dep = 2 - 1;
        let list = vec![
            ("a2a3", 31),
            ("b2b3", 31),
            ("c2c3", 31),
            ("d2d3", 31),
            ("e2e3", 31),
            ("g2g3", 31),
            ("h2h3", 31),
            ("f4f5", 29),
            ("a2a4", 31),
            ("b2b4", 30),
            ("c2c4", 31),
            ("d2d4", 32),
            ("e2e4", 30),
            ("g2g4", 31),
            ("h2h4", 31),
            ("f4e5", 29),
            ("b1a3", 31),
            ("b1c3", 31),
            ("g1f3", 31),
            ("g1h3", 31),
            ("e1f2", 31),
        ];
        for q in list {
            let mut game = get_board(
                "rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 2".to_string(),
            )
            .unwrap();

            let temp_move = parse_move(q.0).unwrap();
            move_piece_no_map(&mut game, &temp_move.0, &temp_move.1, true);

            //rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPPKPP/RNBQ1BNR b kq - 1 2
            //rnbqkbnr/pppp1ppp/8/4p3/5P2/8/PPPPRKPP/1NBQ1BNR b kq - 1 2
            let moves = move_all(game, dep);
            if moves != q.1 {
                let board2 = print_board(&game).unwrap(); //print_board(&game).unwrap();
                println!(">>> {}", board2);
                let game2 = get_board(board2).unwrap();
                let moves2 = move_all(game2, 1);

                println!("{} / {} at {}, check2 {}", moves, q.1, q.0, moves2);
            }

            //break;
        }
        let game =
            get_board("rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPPKPPPP/RNBQ1BNR b kq - 1 2".to_string())
                .unwrap();
        assert_eq!(move_all(game, 2), 726);
        //return;
*/
        let game = get_board(STANDARD_BOARD.to_string()).unwrap();
        assert_eq!(move_all(game, 0), 1);
        assert_eq!(move_all(game, 1), 20);
        assert_eq!(move_all(game, 2), 400);
        assert_eq!(move_all(game, 3), 8902);
        assert_eq!(move_all(game, 4), 197281);
    }
}
