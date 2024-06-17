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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board<G: Graph> {
    pub grid: Vec<Tile>,
    pub graph: G,
    pub num_mines: usize,
}

#[derive(Debug, PartialEq)]
pub struct Solver<'a, Gr: Graph, Ga: Game<Graph = Gr>> {
    board: &'a mut Board<Gr>,
    game: &'a mut Ga,
}

impl<'a, Gr: Graph, Ga: Game<Graph = Gr>> Solver<'a, Gr, Ga> {
    pub fn new(board: &'a mut Board<Gr>, game: &'a mut Ga) -> Self {
        assert!(board.graph == *game.graph());
        Self { board, game }
    }
}

impl<G: Graph> Graph for Board<G> {
    fn for_each_neighbor(&self, pos: usize, callback: impl FnMut(usize)) {
        self.graph.for_each_neighbor(pos, callback)
    }

    fn num_tiles(&self) -> usize {
        self.graph.num_tiles()
    }
}

impl Tile {
    fn needs_flag_fill(&self) -> bool {
        let Hint {
            remaining_mines,
            empties,
            ..
        } = *self
        else {
            return false;
        };
        remaining_mines > 0 && remaining_mines == empties
    }

    fn needs_hint_fill(&self) -> bool {
        let Hint {
            remaining_mines,
            empties,
            ..
        } = *self
        else {
            return false;
        };
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

impl<G: Graph> Board<G> {
    pub fn new(graph: G, num_mines: usize) -> Self {
        Self {
            grid: vec![Empty; graph.num_tiles()],
            graph,
            num_mines,
        }
    }

    pub fn from_grid(grid: Vec<Tile>, graph: G, num_mines: usize) -> Self {
        Self {
            grid,
            graph,
            num_mines,
        }
    }

    pub fn from_game(game: &impl Game<Graph = G>) -> Self
    where
        G: Clone,
    {
        Self::new(game.graph().clone(), game.num_mines())
    }

    pub fn set_tile(&mut self, tile: usize, hint: u8) {
        let mut mines = 0;
        let mut empties = 0;

        self.graph.for_each_neighbor(tile, |n| {
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
    }

    /// Assert that a tile is a hint without making any calls to Game::explore_tile to discover
    /// the tile's value
    pub fn assert_tile(&mut self, tile: usize) {
        if self.grid[tile] != Empty {
            panic!("Attempted to assert a {:?}", self.grid[tile]);
        }

        self.grid[tile] = AssertHint {
            needs_propogate: true,
        };

        self.graph.for_each_neighbor(tile, |n| {
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
            Hint { .. } | AssertHint { .. } => self.graph.for_each_neighbor(tile, |n| {
                if let Hint {
                    ref mut empties, ..
                } = self.grid[n]
                {
                    *empties += 1;
                }
            }),
            Mine { .. } => self.graph.for_each_neighbor(tile, |n| {
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
        self.grid[tile] = Empty;
    }

    pub fn flag_tile(&mut self, tile: usize) {
        if self.grid[tile] != Empty {
            return;
        }

        self.grid[tile] = Mine {
            needs_propogate: true,
        };

        self.graph.for_each_neighbor(tile, |n| {
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

    pub fn is_solved(&self) -> bool {
        self.remaining_empty_tiles() == self.remaining_mines()
    }
}

impl<'a, Gr: Graph, Ga: Game<Graph = Gr>> Solver<'a, Gr, Ga> {
    #[must_use]
    pub fn uncover_tile(&mut self, tile: usize) -> Option<()> {
        if self.board.grid[tile] != Empty {
            return Some(());
        }

        let hint = self.game.explore_tile(tile)?;
        self.board.set_tile(tile, hint);

        Some(())
    }

    pub fn board(&self) -> &Board<Gr> {
        &*self.board
    }

    pub fn game(&self) -> &Ga {
        &*self.game
    }

    pub fn propogate(&mut self, tile: &mut Vec<usize>) {
        let stack = tile;
        let mut neighbors = Vec::with_capacity(8);

        while let Some(loc) = stack.last().copied() {
            let tile = &mut self.board.grid[loc];

            neighbors.clear();
            self.board
                .graph
                .for_each_neighbor(loc, |n| neighbors.push(n));

            if let Mine {
                ref mut needs_propogate,
            } = tile
            {
                *needs_propogate = false;
            }

            if tile.needs_flag_fill() {
                for n in &neighbors {
                    self.board.flag_tile(*n);
                }
            } else if tile.needs_hint_fill() {
                for n in &neighbors {
                    self.uncover_tile(*n).unwrap();
                }
            }

            if let Some(next) = neighbors
                .iter()
                .find(|n| self.board.grid[**n].needs_propogate())
            {
                stack.push(*next);
            } else {
                stack.pop();
            }
        }
    }
}
