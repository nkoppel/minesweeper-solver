mod csp;
// mod solutionset;
mod solve;

pub use std::collections::{HashSet, VecDeque};

use crate::game::*;
pub use csp::*;
pub use solve::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Square {
    Empty,
    Mine { needs_propogate: bool },
    Hint { remaining_mines: u8, empties: u8 },
}

pub use Square::*;

impl Square {
    fn needs_flag_fill(&self) -> bool {
        let Hint { remaining_mines, empties } = *self else { return false };
        remaining_mines > 0 && remaining_mines == empties
    }

    fn needs_hint_fill(&self) -> bool {
        let Hint { remaining_mines, empties } = *self else { return false };
        empties > 0 && remaining_mines == 0
    }

    fn needs_propogate(&self) -> bool {
        match *self {
            Empty => false,
            Mine { needs_propogate } => needs_propogate,
            Hint { .. } => self.needs_flag_fill() || self.needs_hint_fill(),
        }
    }
}

pub struct Solver<G: Game> {
    pub grid: Vec<Square>,
    pub game: G,
}

impl<G: Game> Solver<G> {
    pub fn new(game: G) -> Self {
        Self {
            grid: vec![Empty; game.num_squares()],
            game,
        }
    }

    pub fn uncover_square(&mut self, square: usize) -> Option<()> {
        if self.grid[square] != Empty {
            return Some(());
        }

        let hint = self.game.explore_square(square)?;
        let mut mines = 0;
        let mut empties = 0;

        self.game.for_each_neighbor(square, |n| {
            match self.grid[n] {
                Mine { .. } => mines += 1,
                Empty => empties += 1,
                _ => {}
            }

            if let Hint {
                ref mut empties, ..
            } = self.grid[n]
            {
                *empties -= 1;
            }
        });

        self.grid[square] = Hint {
            remaining_mines: hint - mines,
            empties,
        };

        Some(())
    }

    pub fn flag_square(&mut self, square: usize) {
        if self.grid[square] != Empty {
            return;
        }

        self.grid[square] = Mine {
            needs_propogate: true,
        };

        self.game.for_each_neighbor(square, |n| {
            if let Hint {
                ref mut remaining_mines,
                ref mut empties,
            } = self.grid[n]
            {
                *remaining_mines -= 1;
                *empties -= 1;
            }
        })
    }

    pub fn propogate(&mut self, squares: &mut Vec<usize>) {
        let stack = squares;
        let mut neighbors = Vec::with_capacity(8);

        while let Some(loc) = stack.last().copied() {
            let square = &mut self.grid[loc];

            neighbors.clear();
            self.game.for_each_neighbor(loc, |n| neighbors.push(n));

            if let Mine {
                ref mut needs_propogate,
            } = square
            {
                *needs_propogate = false;
            }

            if square.needs_flag_fill() {
                for n in &neighbors {
                    self.flag_square(*n);
                }
            } else if square.needs_hint_fill() {
                for n in &neighbors {
                    self.uncover_square(*n).unwrap();
                }
            }

            if let Some(next) = neighbors.iter().find(|n| self.grid[**n].needs_propogate()) {
                stack.push(*next);
            } else {
                stack.pop();
            }
        }
    }
}

use std::fmt;

impl fmt::Display for Solver<Game2d> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, x) in self.grid.iter().enumerate() {
            match x {
                Empty => write!(f, ". ")?,
                Mine { .. } => write!(f, "* ")?,
                Hint {
                    remaining_mines, ..
                } => write!(f, "{remaining_mines} ")?,
            }
            if i % self.game.width() == self.game.width() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
