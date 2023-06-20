mod csp;
mod solutionset;
mod solve;

pub use std::collections::{HashSet, VecDeque};

use crate::game::*;
pub use csp::*;
pub use solutionset::*;
pub use solve::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
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

pub use Tile::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Solver<G: Game> {
    pub grid: Vec<Tile>,
    pub game: G,
}

impl Tile {
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

pub fn is_grid_subset_of(subset: &[Tile], set: &[Tile]) -> bool {
    subset
        .iter()
        .zip(set.iter())
        .all(|(s1, s2)| s1.subset_of(s2))
}

impl<G: Game> Solver<G> {
    pub fn new(game: G) -> Self {
        Self {
            grid: vec![Empty; game.num_tiles()],
            game,
        }
    }

    pub fn uncover_tile(&mut self, tile: usize) -> Option<()> {
        if self.grid[tile] != Empty {
            return Some(());
        }

        let hint = self.game.explore_tile(tile)?;
        let mut mines = 0;
        let mut empties = 0;

        self.game.for_each_neighbor(tile, |n| {
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

        self.grid[tile] = Hint {
            hint,
            remaining_mines: hint - mines,
            empties,
        };

        Some(())
    }

    /// Assert that a tile is a hint without making any calls to Game::explore_tile to discover
    /// the tile's value
    pub fn assert_tile(&mut self, tile: usize) {
        if self.grid[tile] != Empty {
            panic!();
        }

        self.grid[tile] = AssertHint {
            needs_propogate: true,
        };

        self.game.for_each_neighbor(tile, |n| {
            if let Hint {
                ref mut empties, ..
            } = self.grid[n]
            {
                *empties -= 1;
            }
        })
    }

    pub fn clear_tile(&mut self, tile: usize) {
        match self.grid[tile] {
            Hint { .. } | AssertHint { .. } => self.game.for_each_neighbor(tile, |n| {
                if let Hint {
                    ref mut empties, ..
                } = self.grid[n]
                {
                    *empties += 1;
                }
            }),
            Mine { .. } => self.game.for_each_neighbor(tile, |n| {
                if let Hint {
                    ref mut remaining_mines,
                    ref mut empties,
                    ..
                } = self.grid[n]
                {
                    *remaining_mines += 1;
                    *empties += 1;
                }
            }),
            Empty => {}
        }
    }

    pub fn flag_tile(&mut self, tile: usize) {
        if self.grid[tile] != Empty {
            return;
        }

        self.grid[tile] = Mine {
            needs_propogate: true,
        };

        self.game.for_each_neighbor(tile, |n| {
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

    pub fn propogate(&mut self, tile: &mut Vec<usize>) {
        let stack = tile;
        let mut neighbors = Vec::with_capacity(8);

        while let Some(loc) = stack.last().copied() {
            let tile = &mut self.grid[loc];

            neighbors.clear();
            self.game.for_each_neighbor(loc, |n| neighbors.push(n));

            if let Mine {
                ref mut needs_propogate,
            } = tile
            {
                *needs_propogate = false;
            }

            if tile.needs_flag_fill() {
                for n in &neighbors {
                    self.flag_tile(*n);
                }
            } else if tile.needs_hint_fill() {
                for n in &neighbors {
                    self.uncover_tile(*n).unwrap();
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
