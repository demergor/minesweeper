use std::io::{self, Write};

use crossterm::{
    self, execute,
    style::{
        Attribute::{Dim, Italic, Reset},
        Print, SetAttribute,
    },
    terminal,
};

use crate::OFFSET;

pub fn query_opts() -> io::Result<(Option<usize>, Option<usize>, Option<usize>)> {
    let mut out = std::io::stdout();
    let term_sz = terminal::size().unwrap();
    let term_height = term_sz.1 as usize;
    let term_width = term_sz.0 as usize;

    if term_width < 1 || term_height < OFFSET {
        panic!("Terminal dimensions too small!");
    }

    let (max_height, max_width) =
        ((term_height as usize - OFFSET), term_width as usize);

    execute!(
        out,
        SetAttribute(Italic),
        SetAttribute(Dim),
        Print(concat!(
            "Ingame controls:\r\n",
            "-- ESCAPE: quit game\r\n",
            "-- Left click: reveal cell\r\n",
            "-- Right click: (un)flag cell\r\n",
            "-- Middle click: (un)assert cell (prevents accidental clicks)\r\n\n",
            "(Non-numeric input terminates)\r\n"
        )),
        SetAttribute(Reset),
    )?;

    let mut buf = String::new();
    print!("Enter the field width (default: {}): ", max_width);
    let _ = out.flush();
    let _ = io::stdin().read_line(&mut buf);

    let Some(width_str) = buf.strip_suffix("\n") else {
        panic!("Couldn't read width!");
    };

    let width = if width_str.is_empty() {
        max_width
    } else if let Ok(width) = width_str.parse::<usize>() {
        width
    } else {
        return Ok((None, None, None));
    };

    buf.clear();
    print!("Enter the field height (default: {}): ", max_height);
    let _ = out.flush();
    let _ = io::stdin().read_line(&mut buf);

    let Some(height_str) = buf.strip_suffix("\n") else {
        panic!("Couldn't read height!");
    };

    let height = if height_str.is_empty() {
        max_height
    } else if let Ok(height) = height_str.parse::<usize>() {
        height
    } else {
        return Ok((Some(width.min(term_width as usize)), None, None));
    };

    buf.clear();
    print!("Enter bomb ratio (default: 10): ");
    let _ = out.flush();
    let _ = io::stdin().read_line(&mut buf);

    let Some(ratio_str) = buf.strip_suffix("\n") else {
        panic!("Couldn't read bomb ratio!");
    };

    let bomb_ratio = if ratio_str.is_empty() {
        10
    } else if let Ok(ratio) = ratio_str.parse::<usize>() {
        ratio
    } else {
        return Ok((
            Some(width.min(term_width as usize)),
            Some(height.min((term_height - OFFSET) as usize)),
            None,
        ));
    };

    Ok((
        Some(width.min(term_width as usize)),
        Some(height.min((term_height - OFFSET) as usize)),
        Some(bomb_ratio),
    ))
}
