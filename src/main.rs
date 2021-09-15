use termcolor::Color;
use vinlag_chess::game_data::*;
use vinlag_chess::logic::*;
use vinlag_chess::parser::*;
use vinlag_chess::render::*;

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
        get_board("rnb1k1nr/pppp1ppp/8/4P3/1p5q/5NP1/PPP1P2P/RNBQK2R w KQkq - 1 4".to_string())
            .unwrap();

    let threats = generate_all_threats(&game, false);
    let moves = generate_valid_moves(&game, &threats, &Position { x: 4, y: 7 });

    render_highlight(
        &game,
        vec![
            (&threats.all_threats, Color::Red),
            (&threats.all_threats_secondary[0], Color::Blue),
            (&threats.all_king_threats, Color::Green),
            (&moves, Color::Rgb(255, 165, 0)),
        ],
    );
    /*
     let pos = Position {
                x: offset_index,
                y: WHITE_SPAWN,
            };
            let mut moves = generate_all_threats(&game, &pos);


            render_highlight(&game, &moves, Color::Red);
            let input = read_input();
            offset_index += 1;

    loop {
        /*let mut highlight = HashSet::new();
        highlight.insert(Position { x: 0, y: 0 });
        game.board[0][0] = PieceData {
            piece: Piece::Pawn,
            is_white: false,
        };*/


        let mut offset_index = 0;
        loop {
            let pos = Position {
                x: offset_index,
                y: WHITE_SPAWN,
            };
            let mut moves = generate_all_threats(&game, &pos);


            render_highlight(&game, &moves, Color::Red);
            let input = read_input();
            offset_index += 1;
        }



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
    }*/
}
