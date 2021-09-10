static BLACK_PICES: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
static WHITE_PICES: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
static BOARD_SIZE: usize = 8;

use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn clear_terminal_color(text: &str) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    write!(&mut stdout, "{}", text)
}

fn change_terminal_colors(text: &str, is_white: bool, is_dark: bool) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(
        ColorSpec::new()
            .set_bold(true)
            .set_fg(if is_white {
                Some(Color::White)
            } else {
                Some(Color::White)
            })
            .set_bg(if is_dark {
                Some(Color::Rgb(24, 26, 27)) //Some(Color::Rgb(233,159,75))
            } else {
                Some(Color::Rgb(49, 53, 55)) // Some(Color::Rgb(171, 113, 59))
            }),
    )?;

    write!(&mut stdout, " {} ", text)
}

fn main() {

    for y in 0..BOARD_SIZE {
        clear_terminal_color("").unwrap();
        print!("{} ", y);
        for x in 0..BOARD_SIZE {
            let is_white = (y + x) % 3 == 1;
            let index = x % 6;
            let pieceChar = if (is_white) {
                WHITE_PICES[index]
            } else {
                BLACK_PICES[index]
            };

            change_terminal_colors(pieceChar, is_white, (y + x) % 2 == 0).unwrap();
        }
        clear_terminal_color("\r\n").unwrap();
    }
    print!(" ");
    for x in 0..BOARD_SIZE {
        print!("  {}", x);
    }

    clear_terminal_color("").unwrap();
}
