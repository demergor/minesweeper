mod field;

use std::io::{self, Write};

use crossterm::{
    self,
    cursor::{Hide, MoveTo, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton,
        MouseEventKind,
    },
    execute,
    style::{
        Attribute::{Bold, Dim, Italic, Reset},
        Color::{Green, Red},
        Print, SetAttribute, SetForegroundColor,
    },
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        );

        eprintln!("{panic_info}");
    }));

    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen,)?;

    let mut buf = String::new();
    let mut run = true;

    while run {
        execute!(
            out,
            Clear(ClearType::All),
            MoveTo(0, 0),
            DisableMouseCapture,
            Show,
        )?;
        terminal::disable_raw_mode()?;
        let term_sz = terminal::size().unwrap();

        if term_sz.0 < 1 || term_sz.1 < 3 {
            panic!("Terminal dimensions too small!");
        }

        execute!(
            out,
            SetAttribute(Italic),
            SetAttribute(Dim),
            Print("(Enter gibberish/nothing to terminate)\n\r"),
            SetAttribute(Reset),
        )?;

        buf.clear();
        print!("Enter the field width (max {}): ", term_sz.0);
        let _ = out.flush();
        let _ = io::stdin().read_line(&mut buf);

        let Some(width_str) = buf.strip_suffix("\n") else {
            panic!("Couldn't read width!");
        };

        let Ok(width) = width_str.parse::<usize>() else {
            break;
        };

        buf.clear();
        print!("Enter the field height (max {}): ", term_sz.1 - 3);
        let _ = out.flush();
        let _ = io::stdin().read_line(&mut buf);

        let Some(height_str) = buf.strip_suffix("\n") else {
            panic!("Couldn't read height!");
        };

        let Ok(height) = height_str.parse::<usize>() else {
            break;
        };

        let mut field = field::Field::new(
            width.min(term_sz.0 as usize),
            height.min((term_sz.1 - 3) as usize),
        );

        terminal::enable_raw_mode()?;
        execute!(
            out,
            EnableMouseCapture,
            Clear(ClearType::All),
            MoveTo(0, 0),
            Hide
        )?;

        let mut status = GameStatus::Running;
        while status == GameStatus::Running {
            execute!(out, Print(&field))?;
            field.undirty();
            match event::read()? {
                Event::Key(key) => {
                    if key.code == event::KeyCode::Esc {
                        run = false;
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    let col: usize = mouse.column as usize;
                    let row: usize = mouse.row as usize;

                    field.hover(col, row);
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            status = field.reveal(col, row);
                        }
                        MouseEventKind::Down(MouseButton::Right) => {
                            field.toggle_flag(col, row);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            if status == GameStatus::Running {
                continue;
            }

            execute!(out, MoveTo(0, 0), Clear(ClearType::All), Print(&field))?;

            if status == GameStatus::Won {
                execute!(
                    out,
                    SetAttribute(Bold),
                    SetForegroundColor(Green),
                    Print("\n\rYOU WON!"),
                    SetAttribute(Reset),
                )?;
            } else {
                execute!(
                    out,
                    SetAttribute(Bold),
                    SetForegroundColor(Red),
                    Print("\n\rYOU LOST!"),
                    SetAttribute(Reset),
                )?;
            }

            execute!(
                out,
                SetAttribute(Italic),
                Print("\n\rPress any key to start over"),
                SetAttribute(Reset),
            )?;

            loop {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(out, LeaveAlternateScreen, DisableMouseCapture, Show)?;

    Ok(())
}

#[derive(Eq, PartialEq)]
pub enum GameStatus {
    Running,
    Over,
    Won,
}
