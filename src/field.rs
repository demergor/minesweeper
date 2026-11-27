use std::fmt;

use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::GameStatus;

pub struct Field {
    contents: Vec<Content>,
    bomb_cnt: usize,
    flag_cnt: usize,
    move_cnt: usize,
    remaining_empty: usize,
    last_hover: (usize, usize),
    dirty: (bool, Vec<(usize, usize)>),
    width: usize,
    height: usize,
}

impl Field {
    pub fn new(width: usize, height: usize, bomb_ratio: usize) -> Field {
        let weights = vec![
            if bomb_ratio >= 100 {
                1
            } else {
                100 - bomb_ratio
            },
            bomb_ratio,
        ];

        let mut contents: Vec<Content> =
            (0..width * height).map(|_| Content::default()).collect();
        let bomb_cnt = Content::populate(&mut contents, &weights);

        Field {
            contents,
            bomb_cnt,
            flag_cnt: 0,
            move_cnt: 0,
            remaining_empty: width * height - bomb_cnt,
            last_hover: (0, 0),
            dirty: (true, Vec::new()),
            width,
            height,
        }
    }

    pub fn reveal(&mut self, x: usize, y: usize) -> GameStatus {
        if x >= self.width || y >= self.height {
            return GameStatus::Running;
        }

        match self.contents[y * self.width + x] {
            Content::Empty(Hardness::Hard, _) | Content::Bomb(Hardness::Hard, _) => {
                return GameStatus::Running;
            }
            Content::Bomb(_, _) => {
                self.contents[y * self.width + x] = Content::RevealedBomb(true);
                self.contents = self
                    .contents
                    .drain(..)
                    .map(|cell| match cell {
                        Content::Bomb(_, _) => Content::RevealedBomb(false),
                        Content::Empty(hardness, _) if hardness != Hardness::None => {
                            Content::FalseFlag
                        }
                        _ => cell,
                    })
                    .collect();
                self.dirty.0 = true;
                self.dirty.1.clear();
                self.move_cnt += 1;

                GameStatus::Over
            }
            Content::Empty(_, _) => {
                self.ripple_reveal(x, y, &mut vec![false; self.width * self.height]);
                self.move_cnt += 1;

                if self.remaining_empty == 0 {
                    GameStatus::Won
                } else {
                    GameStatus::Running
                }
            }
            _ => GameStatus::Running,
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize, hard: bool) -> GameStatus {
        if x >= self.width || y >= self.height {
            return GameStatus::Running;
        }

        let hardness = if hard { Hardness::Hard } else { Hardness::Soft };
        match &mut self.contents[y * self.width + x] {
            Content::Empty(flag, _) | Content::Bomb(flag, _) => match &flag {
                Hardness::None => {
                    *flag = hardness;
                    self.flag_cnt += 1;

                    self.dirty.0 = true;
                    self.dirty.1.push((x, y));
                }
                fh if **fh == hardness => {
                    *flag = Hardness::None;
                    self.flag_cnt -= 1;

                    self.dirty.0 = true;
                    self.dirty.1.push((x, y));
                }
                Hardness::Soft if hardness == Hardness::Hard => {
                    *flag = Hardness::Hard;

                    self.dirty.0 = true;
                    self.dirty.1.push((x, y));
                }
                _ => {}
            },
            _ => {}
        }

        GameStatus::Running
    }

    pub fn hover(&mut self, x: usize, y: usize) {
        match &mut self.contents[self.last_hover.1 * self.width + self.last_hover.0] {
            Content::Empty(_, hover) | Content::Bomb(_, hover) => {
                *hover = false;
                self.dirty.0 = true;
                self.dirty.1.push(self.last_hover);
            }
            _ => {}
        }

        if x >= self.width || y >= self.height {
            return;
        }

        let cell = &mut self.contents[y * self.width + x];
        match cell {
            Content::Empty(_, hover) | Content::Bomb(_, hover) => {
                *hover = true;
                self.last_hover = (x, y);
                self.dirty.1.push((x, y));
                self.dirty.0 = true;
            }
            _ => {}
        };
    }

    pub fn render_height(&self) -> usize {
        self.height + 4
    }

    pub fn undirty(&mut self) {
        self.dirty.0 = false;
    }
}

impl Field {
    fn ripple_reveal(&mut self, x: usize, y: usize, visited: &mut Vec<bool>) {
        if x >= self.width || y >= self.height {
            return;
        }

        if !matches!(self.contents[y * self.width + x], Content::Empty(_, _))
            || visited[y * self.width + x]
        {
            return;
        }

        let mut bomb_cnt = 0;
        let mut neighbors = Vec::with_capacity(8);

        for nx in -1isize..2 {
            for ny in -1isize..2 {
                let nx = x as isize + nx;
                if nx < 0 || nx >= self.width as isize {
                    continue;
                }

                let ny = y as isize + ny;
                if ny < 0 || ny >= self.height as isize {
                    continue;
                }

                if nx == x as isize && ny == y as isize {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                match self.contents[ny * self.width + nx] {
                    Content::Bomb(_, _) | Content::RevealedBomb(_) => bomb_cnt += 1,
                    Content::Empty(_, _) => neighbors.push((nx, ny)),
                    _ => {}
                }
            }
        }

        visited[y * self.width + x] = true;
        self.contents[y * self.width + x] = Content::Revealed(bomb_cnt);
        self.remaining_empty -= 1;
        self.dirty.0 = true;
        self.dirty.1.push((x, y));

        if bomb_cnt != 0 {
            return;
        }

        for (x, y) in neighbors {
            self.ripple_reveal(x, y, visited);
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (dirty, dirties) = &self.dirty;
        if !dirty {
            return Ok(());
        }

        if !dirties.is_empty() {
            for (x, y) in dirties {
                write!(
                    f,
                    "\x1b[{};{}H{}",
                    y + 1,
                    x + 1,
                    self.contents[y * self.width + x]
                )?;
            }

            write!(
                f,
                concat!(
                    "\x1b[{};0H\x1b[2KUnflagged bombs remaining: {}",
                    "\x1b[{};0H\x1b[2KMoves played: {}",
                    "\x1b[{};0H\x1b[2K\x1b[1mProgress: {}%\x1b[m",
                ),
                self.height + 1,
                self.bomb_cnt - self.flag_cnt.min(self.bomb_cnt),
                self.height + 2,
                self.move_cnt,
                self.height + 3,
                100 - ((self.remaining_empty * 100)
                    / ((self.width * self.height) - self.bomb_cnt)),
            )?;

            return Ok(());
        }

        let mut first = true;
        for i in 0..self.height {
            if !first {
                writeln!(f, "\r")?;
            } else {
                first = false;
            }

            for j in 0..self.width {
                write!(f, "{}", self.contents[i * self.width + j])?;
            }
        }

        write!(
            f,
            concat!(
                "\x1b[{};0H\x1b[2KUnflagged bombs remaining: {}",
                "\x1b[{};0H\x1b[2KMoves played: {}",
                "\x1b[{};0H\x1b[2K\x1b[1mProgress: {}%\x1b[m",
            ),
            self.height + 1,
            self.bomb_cnt - self.flag_cnt.min(self.bomb_cnt),
            self.height + 2,
            self.move_cnt,
            self.height + 3,
            100 - ((self.remaining_empty * 100)
                / ((self.width * self.height) - self.bomb_cnt)),
        )
    }
}

#[derive(Eq, PartialEq)]
pub enum Content {
    Empty(Hardness, bool),
    Bomb(Hardness, bool),
    Revealed(usize),
    RevealedBomb(bool),
    FalseFlag,
}

impl Content {
    pub fn populate(collection: &mut [Content], weights: &[usize]) -> usize {
        let mut rng = rand::rng();
        let dist = WeightedIndex::new(weights).unwrap();
        let mut bomb_cnt = 0;

        for cell in collection.iter_mut() {
            *cell = match dist.sample(&mut rng) {
                1 => {
                    bomb_cnt += 1;
                    Content::Bomb(Hardness::None, false)
                }
                _ => Content::Empty(Hardness::None, false),
            }
        }

        bomb_cnt
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::Empty(Hardness::None, false)
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(flag, hovered) | Self::Bomb(flag, hovered) => {
                match (flag, hovered) {
                    (hardness, hovered) if *hardness == Hardness::None => {
                        if *hovered {
                            write!(f, "\x1b[1;2;90;7m@\x1b[m")
                        } else {
                            write!(f, "\x1b[2;90m@\x1b[m")
                        }
                    }
                    (hardness, hovered) => {
                        let color = if *hardness == Hardness::Soft { 31 } else { 33 };
                        if *hovered {
                            write!(f, "\x1b[1;2;{};100mX\x1b[m", color)
                        } else {
                            write!(f, "\x1b[2;{}mX\x1b[m", color)
                        }
                    }
                }
            }
            Self::RevealedBomb(incorrectly_flagged) => {
                if *incorrectly_flagged {
                    write!(f, "\x1b[6m")?;
                }

                write!(f, "\x1b[33;41m#\x1b[m")
            }
            Self::Revealed(remaining) => {
                if *remaining == 0 {
                    return write!(f, " ");
                }

                let color = if *remaining < 2 {
                    36
                } else if *remaining < 3 {
                    33
                } else {
                    31
                };
                write!(f, "\x1b[{}m{}\x1b[m", color, remaining)
            }
            Self::FalseFlag => write!(f, "\x1b[6;31;43mX\x1b[m"),
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum Hardness {
    None,
    Soft,
    Hard,
}
