use crate::game_data::*;
use std::{collections::HashSet, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::game_data::BOARD_SIZE;

const WHITE_PICES: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
const BLACK_PICES: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
const ALPHABET: [&str; BOARD_SIZE] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const REVERSE_BOARD_ON_SWITCH: bool = false;

fn get_char(piece: Piece, is_white: bool) -> &'static str {
    let set = if is_white { WHITE_PICES } else { BLACK_PICES };

    match piece {
        Piece::None => " ",
        Piece::Pawn => set[0],
        Piece::Knight => set[1],
        Piece::Bishop => set[2],
        Piece::Rook => set[3],
        Piece::Queen => set[4],
        Piece::King => set[5],
    }
}

fn clear_terminal_color(text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    write!(&mut stdout, "{}", text).unwrap();
}

fn print_piece(text: &str, fg_color: Color, bg_color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(
            ColorSpec::new()
                .set_bold(true)
                .set_fg(Some(fg_color))
                .set_bg(Some(bg_color)),
        )
        .unwrap();

    write!(&mut stdout, " {}  ", text).unwrap();
}

pub fn render(game: &Game) {
    render_highlight(game, &HashSet::new(), Color::Red);
}

pub fn render_highlight(game: &Game, highlight: &HashSet<Position>, highlight_color: Color) {
    let is_inverted = REVERSE_BOARD_ON_SWITCH && !game.is_white_to_move;

    for y in 0..BOARD_SIZE {
        let display_y = if is_inverted { BOARD_SIZE - y } else { y + 1 };

        clear_terminal_color("");
        print!("{} ", BOARD_SIZE - y);
        for x in 0..BOARD_SIZE {
            let display_x = if is_inverted { BOARD_SIZE - x } else { x + 1 };

            let piece_data = game.board[display_x - 1][display_y - 1];

            let fg_color = if piece_data.is_white {
                Color::White
            } else {
                Color::Rgb(150, 150, 150) // Some(Color::Black)  //Some(Color::Rgb(150,150,150))
            };

            let bg_color = if highlight.contains(&Position { x, y }) {
                highlight_color
            } else {
                let is_dark_square = (display_y + display_x) % 2 == 0;

                if is_dark_square {
                    Color::Rgb(24, 26, 27) // Some(Color::Rgb(233,159,75)) //Some(Color::Rgb(24, 26, 27))
                } else {
                    Color::Rgb(49, 53, 55) // Some(Color::Rgb(171, 113, 59)) //Some(Color::Rgb(49, 53, 55))
                }
            };
            print_piece(
                get_char(piece_data.piece, piece_data.is_white),
                fg_color,
                bg_color,
            );
        }
        clear_terminal_color("\r\n");
    }
    print!("");
    for x in 0..BOARD_SIZE {
        print!("  {} ", ALPHABET[x]);
    }

    clear_terminal_color("\r\n\r\n");
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub fn read_input() -> String {
    let mut text = String::new();
    std::io::stdin().read_line(&mut text).unwrap();
    trim_newline(&mut text);

    text
}