use std::fmt;

use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::GameStatus;

const WEIGHTS: [i32; 2] = [90, 10];
const _COLOR_DIGITS: [i32; 6] = [1, 2, 3, 4, 5, 6];

pub struct Field {
    contents: Vec<Content>,
    bomb_cnt: usize,
    flag_cnt: usize,
    last_hover: (usize, usize),
    dirty: (bool, Vec<(usize, usize)>),
    width: usize,
    height: usize,
}

impl Field {
    pub fn new(width: usize, height: usize) -> Field {
        let mut contents: Vec<Content> =
            (0..width * height).map(|_| Content::default()).collect();
        let bomb_cnt = Content::populate(&mut contents);

        Field {
            contents,
            bomb_cnt,
            flag_cnt: 0,
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
            Content::Bomb(_, _) => {
                self.contents = self
                    .contents
                    .drain(..)
                    .map(|cell| {
                        if let Content::Bomb(_, _) = cell {
                            return Content::RevealedBomb;
                        }

                        cell
                    })
                    .collect();
                self.dirty.0 = true;
                self.dirty.1.clear();

                GameStatus::Over
            }
            Content::Empty(_, _) => {
                self.contents[y * self.width + x] =
                    Content::Revealed(self.surrounding_bombs(x, y));
                // self.ripple_reveal(x, y);
                self.dirty.0 = true;
                self.dirty.1.push((x, y));

                GameStatus::Running
            }
            _ => GameStatus::Running,
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) -> GameStatus {
        if x >= self.width || y >= self.height {
            return GameStatus::Running;
        }

        match &mut self.contents[y * self.width + x] {
            Content::Empty(flagged, _) | Content::Bomb(flagged, _) => {
                if *flagged {
                    *flagged = false;
                    self.flag_cnt -= 1;
                } else {
                    *flagged = true;
                    self.flag_cnt += 1;
                }
            }
            _ => {}
        }

        GameStatus::Running
    }

    pub fn hover(&mut self, x: usize, y: usize) {
        match &mut self.contents[self.last_hover.1 * self.width + self.last_hover.0] {
            Content::Empty(_, hover) | Content::Bomb(_, hover) => *hover = false,
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

    pub fn undirty(&mut self) {
        self.dirty.0 = false;
    }
}

impl Field {
    fn surrounding_bombs(&self, x: usize, y: usize) -> usize {
        if x >= self.width || y >= self.height {
            return 0;
        }

        let mut bomb_cnt = 0;
        for nx in -1isize..2 {
            for ny in -1isize..2 {
                let nx = x as isize + nx;
                if nx < 0 || nx > self.width as isize {
                    continue;
                }

                let ny = y as isize + ny;
                if ny < 0 || ny > self.height as isize {
                    continue;
                }

                if nx == x as isize && ny == y as isize {
                    continue;
                }

                match self.contents[y * self.width + x] {
                    Content::Bomb(_, _) | Content::RevealedBomb => bomb_cnt += 1,
                    _ => {}
                }
            }
        }

        bomb_cnt
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*
        let mut it = COLOR_DIGITS.iter().cycle();
        let width_string = self.width.to_string();

        for i in (0..self.width).rev() {
            if i != 0 && i % width_string.chars().count() == 0 {
                let _ = writeln!(f);
            }

            let digit = i.to_string().chars().next().unwrap();
            write!(f, "\x1b[3{}m{digit}\x1b[0m", it.next().unwrap())?
        }
        */

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
            "\n\rRemaining unflagged bombs: {}",
            self.bomb_cnt - self.flag_cnt.min(self.bomb_cnt)
        )
    }
}

#[derive(Eq, PartialEq)]
pub enum Content {
    Empty(bool, bool),
    Bomb(bool, bool),
    Revealed(usize),
    RevealedBomb,
}

impl Content {
    pub fn populate(collection: &mut [Content]) -> usize {
        let mut rng = rand::rng();
        let dist = WeightedIndex::new(WEIGHTS).unwrap();
        let mut bomb_cnt = 0;

        for cell in collection.iter_mut() {
            *cell = match dist.sample(&mut rng) {
                1 => {
                    bomb_cnt += 1;
                    Content::Bomb(false, false)
                }
                _ => Content::Empty(false, false),
            }
        }

        bomb_cnt
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::Empty(false, false)
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(flagged, hovered) | Self::Bomb(flagged, hovered) => {
                if *hovered {
                    write!(f, "\x1b[2m")?;
                }

                if *flagged {
                    write!(f, "\x1b[2;31mX")?;
                } else {
                    write!(f, "@")?;
                }

                write!(f, "\x1b[0m")
            }
            Self::RevealedBomb => write!(f, "\x1b[33;41m#\x1b[0m"),
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
                write!(f, "\x1b[{}m{}\x1b[0m", color, remaining)
            }
        }
    }
}
