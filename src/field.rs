use std::fmt;

use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::GameStatus;

const WEIGHTS: [i32; 2] = [90, 10];
const COLOR_DIGITS: [i32; 6] = [1, 2, 3, 4, 5, 6];

pub struct Field {
    contents: Vec<Content>,
    bomb_cnt: usize,
    flag_cnt: usize,
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
            width,
            height,
        }
    }

    pub fn reveal(&mut self, x: usize, y: usize) -> GameStatus {
        if x >= self.width || y >= self.height {
            print!("\x1b[{}HInvalid move! x: {x}, y: {y}", self.height + 1);
        }

        match self.contents[y * self.width + x] {
            Content::Bomb(_) => {
                self.contents = self
                    .contents
                    .iter()
                    .map(|cell| {
                        if let Content::Bomb(_) = *cell {
                            Content::RevealedBomb
                        } else {
                            Content::Revealed(0)
                        }
                    })
                    .collect();

                GameStatus::Over
            }
            Content::Empty(_) => {
                self.contents[y * self.width + x] =
                    Content::Revealed(self.surrounding_bombs(x, y));

                GameStatus::Running
            }
            _ => GameStatus::Running
        }
    }
}

impl Field {
    fn surrounding_bombs(&self, x: usize, y: usize) -> usize {
        todo!()
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = COLOR_DIGITS.iter().cycle();
        let width_string = self.width.to_string();
        
        for i in (0..self.width).rev() {
            if i != 0 && i % width_string.chars().count() == 0 {
                let _ = writeln!(f);
            }

            let digit = i.to_string().chars().next().unwrap();
            write!(f, "\x1b[3{}m{digit}\x1b[0m", it.next().unwrap())?
        }

        let mut first = true;
        for i in 0..self.height {
            if !first {
                writeln!(f)?;
            } else {
                first = false;
            }

            for j in 0..self.width {
                write!(f, "{}", self.contents[i * self.width + j])?;
            }
        }

        write!(f, "\nRemaining unflagged bombs: {}", self.bomb_cnt)
    }
}

#[derive(Eq, PartialEq)]
pub enum Content {
    Empty(bool),
    Bomb(bool),
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
                    Content::Bomb(false)
                }
                _ => Content::Empty(false),
            }
        }

        bomb_cnt
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::Empty(false)
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(flagged) | Self::Bomb(flagged) => {
                if *flagged {
                    write!(f, "\x1b[2;31mX\x1b[0m")
                } else {
                    write!(f, "\x1b[2m@\x1b[0m")
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
