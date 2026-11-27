mod cli;
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

use crate::cli::query_opts;

const OFFSET: usize = 6;

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

    let program_start = std::time::Instant::now();
    let mut out = io::stdout();
    execute!(out, EnterAlternateScreen,)?;

    let mut run = true;
    while run {
        execute!(
            out,
            MoveTo(0, 0),
            Clear(ClearType::All),
            DisableMouseCapture,
            Show,
        )?;
        terminal::disable_raw_mode()?;
        let (Some(width), Some(height), Some(bomb_ratio)) = query_opts()? else {
            break;
        };

        let mut field = field::Field::new(width, height, bomb_ratio);

        terminal::enable_raw_mode()?;
        execute!(
            out,
            Hide,
            EnableMouseCapture,
            MoveTo(0, 0),
            Clear(ClearType::All),
            Print(&field),
        )?;

        let mut status = GameStatus::Running;
        let start = std::time::Instant::now();

        while status == GameStatus::Running {
            let (elapsed_hours, elapsed_minutes, elapsed_secs) =
                seconds_to_hms(start.elapsed().as_secs());
            print!(
                "\x1b[{}H{}h {}m {}s",
                field.render_height(),
                elapsed_hours,
                elapsed_minutes,
                elapsed_secs,
            );
            out.flush()?;

            if !event::poll(std::time::Duration::from_millis(1000))? {
                continue;
            }

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

                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            status = field.reveal(col, row);
                        }
                        MouseEventKind::Down(MouseButton::Right) => {
                            field.toggle_flag(col, row, false);
                        }
                        MouseEventKind::Down(MouseButton::Middle) => {
                            field.toggle_flag(col, row, true);
                        }
                        MouseEventKind::Moved => field.hover(col, row),
                        _ => {}
                    }
                }
                _ => {}
            }

            execute!(out, MoveTo(0, 0), Print(&field))?;
            field.undirty();

            if status == GameStatus::Running {
                continue;
            }

            if status == GameStatus::Won {
                execute!(
                    out,
                    SetAttribute(Bold),
                    SetForegroundColor(Green),
                    Print("\r\n\nYOU WON!"),
                    SetAttribute(Reset),
                )?;
            } else {
                execute!(
                    out,
                    SetAttribute(Bold),
                    SetForegroundColor(Red),
                    Print("\r\n\nYOU LOST!"),
                    SetAttribute(Reset),
                )?;
            }

            execute!(
                out,
                SetAttribute(Italic),
                SetAttribute(Dim),
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
    let (h, m, s) = seconds_to_hms(program_start.elapsed().as_secs());
    println!("Your minesweeper session lasted {h} hours, {m} minutes, {s} seconds");

    Ok(())
}

#[derive(Eq, PartialEq)]
pub enum GameStatus {
    Running,
    Over,
    Won,
}

fn seconds_to_hms(mut seconds: u64) -> (u64, u64, u64) {
    let mut minutes = seconds / 60;
    seconds %= 60;
    let hours = minutes / 60;
    minutes %= 60;

    (hours, minutes, seconds)
}
