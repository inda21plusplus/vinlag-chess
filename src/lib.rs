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
    fn run_deep_test_undo() {
        let game = get_board(STANDARD_BOARD.to_string()).unwrap();
        //println!("TIME {}ms", move_all(game, 3).1 / 1_000_000);
        assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 20);
        assert_eq!(move_all(game, 2).0, 400);
        assert_eq!(move_all(game, 3).0, 8902);
        assert_eq!(move_all(game, 4).0, 197281);

        //627.90s and it works!!
        //assert_eq!(move_all(game, 5, 0).0, 4865609);
    }

    #[test]
    fn run_deep_test_2() {
        let game = get_board("rnbqkbnr/ppp1p2p/6p1/3p1p2/6P1/5P1N/PPPPP1BP/RNBQK2R b KQkq - 1 4".to_string()).unwrap();
        //println!("TIME {}ms", move_all(game, 3).1 / 1_000_000);
       // assert_eq!(move_all(game, 0).0, 1);
        assert_eq!(move_all(game, 1).0, 27);
        assert_eq!(move_all(game, 2).0, 671);
       // assert_eq!(move_all(game, 3).0, 8902);
       // assert_eq!(move_all(game, 3).0, 19218);
    }
}
