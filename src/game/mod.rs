use rand::{prelude::*, rng};

mod game2d;
mod precomputed_graph;

pub use game2d::*;
pub use precomputed_graph::*;

use crate::bitset::BitSet;

pub trait Graph: Clone + PartialEq + Eq {
    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> + '_;

    fn num_tiles(&self) -> usize;
}

pub trait Game {
    type Graph: Graph;

    fn graph(&self) -> &Self::Graph;
    #[must_use]
    fn explore_tile(&mut self, pos: usize) -> Option<u8>;
    fn num_mines(&self) -> usize;
}

impl<G: Game + Clone + Eq> Graph for G {
    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> + '_ {
        self.graph().neighbors(pos)
    }

    fn num_tiles(&self) -> usize {
        self.graph().num_tiles()
    }
}

// returns n unique random numbers from 0 to max - 1
pub(crate) fn n_unique_random(
    max: usize,
    n: usize,
    rng: &mut impl Rng,
) -> impl Iterator<Item = usize> + '_ {
    if n > max {
        panic!("Cannot generate {n} random numbers from 0..{max}");
    }

    let mut vec: Vec<usize> = (0..max).collect();

    (0..n).map(move |_| vec.swap_remove(rng.random_range(0..vec.len())))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartType {
    Unsafe,
    Safe,
    SafeNeighborhood,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InternalGame<G: Graph> {
    pub grid: Option<BitSet>,
    start_type: StartType,
    num_mines: usize,
    graph: G,
}

impl<G: Graph> InternalGame<G> {
    pub fn new(num_mines: usize, start_type: StartType, graph: G) -> Self {
        Self {
            grid: None,
            num_mines,
            start_type,
            graph,
        }
    }

    pub fn from_game(start_type: StartType, game: &impl Game<Graph = G>) -> Self
    where
        G: Clone,
    {
        Self::new(game.num_mines(), start_type, game.graph().clone())
    }

    pub fn from_grid(grid: BitSet, graph: G) -> Self {
        Self {
            num_mines: grid.count_ones(),
            grid: Some(grid),
            start_type: StartType::Unsafe,
            graph,
        }
    }

    fn explore_tile_inner(&self, grid: &BitSet, pos: usize) -> Option<u8> {
        if grid[pos] {
            return None;
        }

        let mut count = 0;

        for pos2 in self.neighbors(pos) {
            count += grid[pos2] as u8;
        }

        Some(count)
    }

    fn generate_grid(&self, pos: usize) -> BitSet {
        let mut safe: Vec<usize> = Vec::new();

        match self.start_type {
            StartType::Unsafe => {}
            StartType::Safe => safe.push(pos),
            StartType::SafeNeighborhood => {
                safe.push(pos);
                safe.extend(self.neighbors(pos));
            }
        }

        let mut allowed: Vec<usize> = (0..self.num_tiles()).collect();

        safe.sort_unstable();
        for s in safe.iter().rev() {
            allowed.swap_remove(*s);
        }

        let (mines, _) = allowed.partial_shuffle(&mut rng(), self.num_mines);

        BitSet::from_iter(mines.iter().copied(), self.num_tiles())
    }
}

impl<G: Graph> Game for InternalGame<G> {
    type Graph = G;

    fn graph(&self) -> &Self::Graph {
        &self.graph
    }

    fn explore_tile(&mut self, pos: usize) -> Option<u8> {
        if self.grid.is_none() {
            self.grid = Some(self.generate_grid(pos))
        }

        self.explore_tile_inner(self.grid.as_ref().unwrap(), pos)
    }

    fn num_mines(&self) -> usize {
        self.num_mines
    }
}
