use std::fmt;

use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::GameStatus;

pub struct Field {
    contents: Vec<Content>,
    bomb_cnt: usize,
    flag_cnt: usize,
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
                self.ripple_reveal(x, y, &mut vec![false; self.width * self.height]);
                if self.remaining_empty == 0 {
                    GameStatus::Won
                } else {
                    GameStatus::Running
                }
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
                    Content::Bomb(_, _) | Content::RevealedBomb => bomb_cnt += 1,
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
                "\x1b[{};0H\x1b[2KRemaining unflagged bombs: {}",
                self.height + 1,
                self.bomb_cnt - self.flag_cnt.min(self.bomb_cnt)
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
    pub fn populate(collection: &mut [Content], weights: &[usize]) -> usize {
        let mut rng = rand::rng();
        let dist = WeightedIndex::new(weights).unwrap();
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
                match (flagged, hovered) {
                    (true, hovered) => {
                        if *hovered {
                            write!(f, "\x1b[1;100m")?;
                        }

                        write!(f, "\x1b[31mX\x1b[0m")
                    }
                    (false, hovered) => {
                        if *hovered {
                            write!(f, "\x1b[1;7m")?;
                        }

                        write!(f, "\x1b[2;90m@\x1b[0m")
                    }
                }
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
