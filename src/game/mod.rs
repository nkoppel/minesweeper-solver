use bitvec::prelude::*;
use itertools::Itertools;
use rand::prelude::*;

mod game2d;

pub use game2d::*;

pub trait Graph {
    fn for_each_neighbor(&self, pos: usize, callback: impl FnMut(usize));
    fn num_tiles(&self) -> usize;
}

pub trait Game {
    type Graph: Graph;

    fn graph(&self) -> &Self::Graph;
    fn explore_tile(&mut self, pos: usize) -> Option<u8>;
    fn num_mines(&self) -> usize;
}

impl<G: Game> Graph for G {
    fn for_each_neighbor(&self, pos: usize, callback: impl FnMut(usize)) {
        self.graph().for_each_neighbor(pos, callback)
    }

    fn num_tiles(&self) -> usize {
        self.graph().num_tiles()
    }
}

// returns n unique randome numbers from 0 to max - 1
pub(crate) fn n_unique_random(
    max: usize,
    n: usize,
    rng: &mut impl Rng,
) -> impl Iterator<Item = usize> + '_ {
    if n > max {
        panic!("Cannot generate {n} random numbers from 0..{max}");
    }

    let mut vec: Vec<usize> = (0..max).collect();

    (0..n).map(move |_| vec.swap_remove(rng.gen_range(0..vec.len())))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartType {
    Unsafe,
    Safe,
    SafeNeighborhood,
}

#[derive(Clone, Debug)]
pub struct InternalGame<G: Graph> {
    pub grid: Option<BitVec>,
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
        where G: Clone
    {
        Self::new(game.num_mines(), start_type, game.graph().clone())
    }

    fn explore_tile_inner(&self, grid: &BitVec, pos: usize) -> Option<u8> {
        (!grid[pos]).then(|| {
            let mut count = 0;

            self.for_each_neighbor(pos, |pos2| {
                count += grid[pos2] as u8;
            });

            count
        })
    }

    fn generate_grid(&self, pos: usize) -> BitVec {
        let mut out = bitvec![usize, Lsb0; 0; self.num_tiles()];
        let mut rng = thread_rng();

        if self.start_type == StartType::Unsafe {
            for i in n_unique_random(self.num_tiles(), self.num_mines(), &mut rng) {
                out.set(i, true);
            }

            return out;
        }

        let mut allowed = bitvec![usize, Lsb0; 1; self.num_tiles()];

        allowed.set(pos, false);
        if self.start_type == StartType::SafeNeighborhood {
            self.for_each_neighbor(pos, |pos2| {
                allowed.set(pos2, false);
            })
        }

        let mut allowed = allowed.iter_ones().collect_vec();

        for i in (0..self.num_mines).map(|_| allowed.swap_remove(rng.gen_range(0..allowed.len()))) {
            out.set(i, true);
        }

        out
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
