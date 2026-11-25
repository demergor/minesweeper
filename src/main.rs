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
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> io::Result<()> {
    let mut out = io::stdout();
    execute!(out, Hide, EnterAlternateScreen,)?;

    let mut buf = String::new();
    let mut run = true;

    while run {
        execute!(
            out,
            Clear(ClearType::All),
            MoveTo(0, 0),
            DisableMouseCapture
        )?;
        terminal::disable_raw_mode()?;
        buf.clear();
        print!("Enter the field width: ");
        let _ = out.flush();
        let _ = io::stdin().read_line(&mut buf);

        let Some(width_str) = buf.strip_suffix("\n") else {
            panic!("Couldn't read width!");
        };

        let Ok(width) = width_str.parse::<usize>() else {
            panic!("Couldn't read width!");
        };

        buf.clear();
        print!("Enter the field height: ");
        let _ = out.flush();
        let _ = io::stdin().read_line(&mut buf);

        let Some(height_str) = buf.strip_suffix("\n") else {
            panic!("Couldn't read height!");
        };

        let Ok(height) = height_str.parse::<usize>() else {
            panic!("Couldn't read height!");
        };

        let mut field = field::Field::new(width, height);
        terminal::enable_raw_mode()?;
        execute!(out, EnableMouseCapture, Clear(ClearType::All), MoveTo(0, 0))?;

        let mut status = GameStatus::Running;
        while status == GameStatus::Running {
            execute!(out, Print(&field))?;
            field.undirty();
            match event::read()? {
                Event::Key(key) => {
                    if key.code == event::KeyCode::Esc {
                        run = false;
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

            if status != GameStatus::Over {
                continue;
            }

            execute!(
                out,
                MoveTo(0, 0),
                Clear(ClearType::All),
                Print(&field),
                Print("Press any key to start over")
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
}
