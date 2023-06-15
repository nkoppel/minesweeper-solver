mod csp;
mod solutionset;
mod solve;

pub use std::collections::{HashSet, VecDeque};

use crate::game::*;
pub use csp::*;
pub use solve::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Square {
    Empty,
    Mine {
        needs_propogate: bool,
    },
    AssertHint {
        needs_propogate: bool,
    },
    Hint {
        hint: u8,
        remaining_mines: u8,
        empties: u8,
    },
}

pub use Square::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Solver<G: Game> {
    pub grid: Vec<Square>,
    pub game: G,
}

impl Square {
    fn needs_flag_fill(&self) -> bool {
        let Hint { remaining_mines, empties, .. } = *self else { return false };
        remaining_mines > 0 && remaining_mines == empties
    }

    fn needs_hint_fill(&self) -> bool {
        let Hint { remaining_mines, empties, .. } = *self else { return false };
        empties > 0 && remaining_mines == 0
    }

    fn needs_propogate(&self) -> bool {
        match *self {
            Empty => false,
            Mine { needs_propogate } => needs_propogate,
            AssertHint { needs_propogate } => needs_propogate,
            Hint { .. } => self.needs_flag_fill() || self.needs_hint_fill(),
        }
    }

    pub fn subset_of(&self, other: &Self) -> bool {
        match (self, other) {
            (_, Empty) => true,
            (Hint { .. }, AssertHint { .. }) => true,
            (Hint { hint: h1, .. }, Hint { hint: h2, .. }) => h1 == h2,
            _ => false,
        }
    }
}

pub fn grid_subset_of(subset: &[Square], set: &[Square]) -> bool {
    subset
        .iter()
        .zip(set.iter())
        .all(|(s1, s2)| s1.subset_of(s2))
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
            hint,
            remaining_mines: hint - mines,
            empties,
        };

        Some(())
    }

    /// Assert that a square is a hint without making any calls to Game::explore_square to discover
    /// the square's value
    pub fn assert_square(&mut self, square: usize) {
        if self.grid[square] != Empty {
            panic!();
        }

        self.grid[square] = AssertHint {
            needs_propogate: true,
        };

        self.game.for_each_neighbor(square, |n| {
            if let Hint {
                ref mut empties, ..
            } = self.grid[n]
            {
                *empties -= 1;
            }
        })
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
                ..
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
                AssertHint { .. } => write!(f, "? ")?,
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
