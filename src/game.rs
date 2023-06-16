use bitvec::prelude::*;
use rand::prelude::*;

pub trait Game {
    fn for_each_neighbor(&self, pos: usize, callback: impl FnMut(usize));
    fn explore_square(&mut self, pos: usize) -> Option<u8>;
    fn num_squares(&self) -> usize;
    fn num_mines(&self) -> usize;
}

pub trait InternalGame: Game + Clone {
    fn set_grid(&mut self, grid: BitVec);

    fn with_grid(&self, grid: BitVec) -> Self {
        let mut out = self.clone();
        out.set_grid(grid);
        out
    }
}

#[derive(Clone, Debug)]
pub struct Game2d {
    grid: BitVec,
    width: usize,
    num_mines: usize,
    neighbors: Vec<(isize, isize)>,
}

#[rustfmt::skip]
pub const MOORE_NEIGHBORHOOD: [(isize, isize); 8] =
    [
        (-1, -1), ( 0, -1), ( 1, -1),
        (-1,  0),           ( 1,  0),
        (-1,  1), ( 0,  1), ( 1,  1)
    ];

#[rustfmt::skip]
pub const VON_NEUMANN_NEIGHBORHOOD: [(isize, isize); 4] =
    [
                  ( 0, -1),
        (-1,  0),           ( 1,  0),
                  ( 0,  1),
    ];

#[rustfmt::skip]
pub const KNIGHT_NEIGHBORHOOD: [(isize, isize); 8] =
    [
               (-1, -2),   ( 1, -2),
        (-2, -1),                 ( 2, -1),

        (-2,  1),                 ( 2,  1),
               (-1,  2),   ( 1,  2),
    ];

fn valid_neighbors_2d(
    neighbors: impl Iterator<Item = (isize, isize)>,
    (w, h): (usize, usize),
    (x, y): (usize, usize),
) -> impl Iterator<Item = (usize, usize)> {
    neighbors.filter_map(move |(xi, yi)| {
        let x2 = (x as isize + xi) as usize;
        let y2 = (y as isize + yi) as usize;

        ((0..w).contains(&x2) && (0..h).contains(&y2)).then_some((x2, y2))
    })
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

impl Game2d {
    pub fn new(
        width: usize,
        height: usize,
        num_mines: usize,
        neighbors: Vec<(isize, isize)>,
        rng: &mut impl Rng,
    ) -> Self {
        let size = width * height;
        let mut grid = bitvec![usize, Lsb0; 0; size];

        for i in n_unique_random(size, num_mines, rng) {
            grid.set(i, true);
        }

        Self {
            grid,
            width,
            num_mines,
            neighbors,
        }
    }

    /// Constructs a game using the first width * height elements of 'grid'.
    pub fn with_grid(
        width: usize,
        height: usize,
        neighbors: Vec<(isize, isize)>,
        grid: impl Iterator<Item = bool> + Clone,
    ) -> Self {
        let grid = grid.take(width * height);
        let num_mines = grid.clone().map(|x| x as usize).sum();
        let grid = grid.collect::<BitVec>();

        assert_eq!(grid.len(), width * height);

        Self {
            grid,
            width,
            num_mines,
            neighbors,
        }
    }

    pub fn from_2d_grid(neighbors: Vec<(isize, isize)>, grid: &[Vec<bool>]) -> Self {
        let width = grid[0].len();
        let height = grid.len();
        let iter = grid.iter().flatten().copied();

        Self::with_grid(width, height, neighbors, iter)
    }

    fn neighbors_iter(&self, pos: usize) -> impl Iterator<Item = usize> + '_ {
        let size = (self.width, self.grid.len() / self.width);
        let pos = (pos % self.width, pos / self.width);

        valid_neighbors_2d(self.neighbors.iter().copied(), size, pos)
            .map(move |(x, y)| x + y * self.width)
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

impl Game for Game2d {
    fn for_each_neighbor(&self, pos: usize, callback: impl FnMut(usize)) {
        self.neighbors_iter(pos).for_each(callback)
    }

    fn explore_square(&mut self, pos: usize) -> Option<u8> {
        (!self.grid[pos]).then(|| self.neighbors_iter(pos).map(|i| self.grid[i] as u8).sum())
    }

    fn num_squares(&self) -> usize {
        self.grid.len()
    }

    fn num_mines(&self) -> usize {
        self.num_mines
    }
}

impl InternalGame for Game2d {
    fn set_grid(&mut self, grid: BitVec) {
        self.grid = grid;
    }
}

use std::fmt;

impl fmt::Display for Game2d {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, x) in self.grid.iter().enumerate() {
            if *x {
                write!(f, "* ")?;
            } else {
                write!(f, ". ")?;
            }
            if i % self.width == self.width - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SafeStartGame<G: InternalGame> {
    pub inner: G,
    started: bool,
}

impl<G: InternalGame> SafeStartGame<G> {
    pub fn new(game: G) -> Self {
        Self {
            inner: game,
            started: false,
        }
    }
}

impl<G: InternalGame> Game for SafeStartGame<G> {
    fn for_each_neighbor(&self, pos: usize, callback: impl FnMut(usize)) {
        self.inner.for_each_neighbor(pos, callback)
    }

    fn explore_square(&mut self, pos: usize) -> Option<u8> {
        if self.started {
            return self.inner.explore_square(pos);
        }

        let mut allowed_squares = bitvec![usize, Lsb0; 1; self.num_squares()];

        allowed_squares.set(pos, false);
        self.for_each_neighbor(pos, |n| {
            allowed_squares.set(n, false);
        });

        let allowed_squares: Vec<usize> = allowed_squares.iter_ones().collect();
        let mut grid = bitvec![usize, Lsb0; 0; self.num_squares()];

        for i in n_unique_random(allowed_squares.len(), self.num_mines(), &mut thread_rng()) {
            grid.set(allowed_squares[i], true);
        }

        self.started = true;
        self.inner.set_grid(grid);
        Some(0)
    }

    fn num_squares(&self) -> usize {
        self.inner.num_squares()
    }

    fn num_mines(&self) -> usize {
        self.inner.num_mines()
    }
}

impl<G: InternalGame> InternalGame for SafeStartGame<G> {
    fn set_grid(&mut self, grid: BitVec) {
        self.inner.set_grid(grid)
    }
}

impl<G: InternalGame + fmt::Display> fmt::Display for SafeStartGame<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
