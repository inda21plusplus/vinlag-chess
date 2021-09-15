use vinlag_chess::render::*;
use vinlag_chess::parser::*;
use vinlag_chess::logic::*;
use vinlag_chess::game_data::*;

fn main() {
    //chcp 65001
    //std::process::Command::new("chcp 65001");
    //std::process::Command::new("clear");
    //std::process::Command::new("cls");
    println!("==================================");

    //both castle
    //rnbq1rk1/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 w - - 4 6

    // one castle
    //rnbqk2r/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 b kq - 3 5

    // no castle
    // rnbqk2r/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQK2R w KQkq - 2 5

    let mut game =
        get_board("rnbq1rk1/pppp2pp/3b1n2/4pp2/4PP2/3B1N2/PPPP2PP/RNBQ1RK1 w - - 4 6".to_string())
            .unwrap();

    let game_print = print_board(&game);
    println!("{}", game_print.unwrap());

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
