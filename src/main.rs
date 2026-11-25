mod field;

use std::io::{self, Write};

fn main() {
    let mut buf = String::new();
    print!("\x1b[?1049h\x1b[?1006h\x1b[H\x1b[2J");
    print!("Enter the field width: ");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut buf);

    let Some(width_str) = buf.strip_suffix("\n") else {
        panic!("Couldn't read width!");
    };

    let Ok(width) = width_str.parse::<usize>() else {
        panic!("Couldn't read width!");
    };

    let Some(height_str) = buf.strip_suffix("\n") else {
        panic!("Couldn't read width!");
    };

    let Ok(height) = height_str.parse::<usize>() else {
        panic!("Couldn't read height!");
    };

    print!("Enter the field height: ");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut buf);

    let field = field::Field::new(width, height);
    loop {
        print!("\x1b[H\x1b[2J");
        buf.clear();
        let _ = io::stdin().read_line(&mut buf);
        if buf == "quit\n" {
            break;
        }

        println!("{field}");
        let _ = io::stdout().flush();
    }

    print!("\x1b[?1049l\x1b[?1006l");
}

pub enum GameStatus {
    Running,
    Over,
}
