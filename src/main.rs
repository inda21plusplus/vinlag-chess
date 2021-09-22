use vinlag_chess::game_data::*;
use vinlag_chess::logic::*;
use vinlag_chess::parser::*;
use vinlag_chess::render::*;

fn main() {
    //chcp 65001

    let mut game_board = init_game_board(STANDARD_BOARD.to_string()).unwrap();
    loop {
        render(&game_board.game);

        let threats = get_threats(&game_board);
        let win_status = get_game_state(&game_board, &threats, true);
        match win_status {
            WinStatus::BlackWon => {
                println!("Black won!");
                break;
            }
            WinStatus::WhiteWon => {
                println!("White won!");
                break;
            }
            WinStatus::Tie => {
                println!("Tie!");
                break;
            }
            WinStatus::Nothing => {
                // nothing intresting
            }
        }

        loop {
            // for GUI you can use get_all_valid_moves(&game_board, &threats, piece_position) for a preview
            // you can directily use Position for GUI
            let mut player_move = None;
            while player_move.is_none() {
                player_move = parse_move(&read_input());
                if player_move.is_none() {
                    println!("Invalid input")
                }
            }
            let (move_start, move_end) = player_move.unwrap();

            // use promote_pawn if auto_promote is set to false
            if move_piece(&mut game_board, move_start, move_end, &threats, true) {
                break;
            } else {
                println!("Invalid move")
            }
        }
    }
}
