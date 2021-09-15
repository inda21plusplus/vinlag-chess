pub mod game_data;
pub mod render;
pub mod parser;
pub mod logic;

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
            let pos = match parse_move(&input) {
                Some(t) => t,
                None => return None,
            };
            let flags = move_piece(&mut game, &pos.0, &pos.1, true);
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
    const STANDARD_BOARD : &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    #[test]
    fn fen_test_no_castle() {
        let str = "rnbqk2r/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQK2R w KQkq - 2 5";
        assert_eq!(str, load_board(str, Vec::new()).unwrap().0);
    }

    #[test]
    fn fen_test_one_castle() {
        let str = "rnbqk2r/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 b kq - 3 5";
        let q = load_board(str, Vec::new());
        assert_eq!(str, q.unwrap().0);
    }

    #[test]
    fn fen_test_both_castle() {
        let str = "rnbq1rk1/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 w - - 4 6";
        assert_eq!(str, load_board(str, Vec::new()).unwrap().0);
    }

    #[test]
    fn simple_move() {
        let str = STANDARD_BOARD;
        assert_eq!("rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2", load_board(str, vec!["e2e4","f7f5","e4f5"]).unwrap().0);
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
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 4 3", load_board(str, vec!["b1c3","b8c6","c3b1","c6b8"]).unwrap().0);
    }
}