use crate::game::*;

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

impl<G: Graph> Graph for Board<G> {
    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> + '_ {
        self.graph.neighbors(pos)
    }

    fn num_tiles(&self) -> usize {
        self.graph.num_tiles()
    }
}

impl Tile {
    pub fn needs_flag_fill(&self) -> bool {
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

    pub fn needs_hint_fill(&self) -> bool {
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

    pub fn needs_propogate(&self) -> bool {
        match *self {
            Empty => false,
            Mine { needs_propogate } => needs_propogate,
            AssertHint { needs_propogate } => needs_propogate,
            Hint { .. } => self.needs_flag_fill() || self.needs_hint_fill(),
        }
    }

    pub fn subset_of(&self, other: &Self) -> bool {
        match other {
            Empty => true,
            Mine { .. } => matches!(self, Mine { .. }),
            AssertHint { .. } => matches!(self, Hint { .. } | AssertHint { .. }),
            Hint { hint: h1, .. } => matches!(self, Hint { hint: h2, .. } if h1 == h2),
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

    #[must_use]
    pub fn set_tile(&mut self, tile: usize, hint: u8) -> Option<()> {
        self.clear_tile(tile);

        let mut mines = 0;
        let mut empties = 0;

        for n in self.graph.neighbors(tile) {
            match self.grid[n] {
                Mine { .. } => mines += 1,
                Empty => empties += 1,
                Hint {
                    ref mut empties, ..
                } => *empties -= 1,
                _ => {}
            }
        }

        if mines > hint || empties < hint - mines {
            return None;
        }

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
            panic!("Attempted to assert a {:?}", self.grid[tile]);
        }

        self.grid[tile] = AssertHint {
            needs_propogate: true,
        };

        for n in self.graph.neighbors(tile) {
            if let Hint {
                ref mut empties, ..
            } = self.grid[n]
            {
                *empties -= 1;
            }
        }
    }

    pub fn clear_tile(&mut self, tile: usize) {
        match self.grid[tile] {
            Hint { .. } | AssertHint { .. } => {
                for n in self.graph.neighbors(tile) {
                    if let Hint {
                        ref mut empties, ..
                    } = self.grid[n]
                    {
                        *empties += 1;
                    }
                }
                self.grid[tile] = Empty;
            }
            Mine { .. } => {
                for n in self.graph.neighbors(tile) {
                    if let Hint {
                        ref mut remaining_mines,
                        ref mut empties,
                        ..
                    } = self.grid[n]
                    {
                        *remaining_mines += 1;
                        *empties += 1;
                    }
                }
                self.grid[tile] = Empty;
            }
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

        for n in self.graph.neighbors(tile) {
            if let Hint {
                ref mut remaining_mines,
                ref mut empties,
                ..
            } = self.grid[n]
            {
                *remaining_mines -= 1;
                *empties -= 1;
            }
        }
    }

    pub fn remaining_mines(&self) -> usize {
        let placed_mines = self
            .grid
            .iter()
            .filter(|s| matches!(s, Mine { .. }))
            .count();

        self.num_mines - placed_mines
    }

    pub fn remaining_empty_tiles(&self) -> usize {
        self.grid.iter().filter(|s| matches!(s, Empty)).count()
    }

    pub fn is_solved(&self) -> bool {
        self.remaining_empty_tiles() == self.remaining_mines()
    }

    pub fn subset_of(&self, other: &Self) -> bool {
        self.num_mines == other.num_mines && is_grid_subset_of(&self.grid, &other.grid)
    }

    pub fn normalize(&self) -> Self {
        let grid = self
            .grid
            .iter()
            .map(|tile| match *tile {
                Mine { needs_propogate } => AssertHint { needs_propogate },
                Hint {
                    remaining_mines: 0, ..
                } => AssertHint {
                    needs_propogate: false,
                },
                Hint {
                    remaining_mines,
                    empties,
                    ..
                } => Hint {
                    hint: remaining_mines,
                    remaining_mines,
                    empties,
                },
                tile => tile,
            })
            .collect();

        Self {
            grid,
            graph: self.graph.clone(),
            num_mines: self.remaining_mines(),
        }
    }
}
