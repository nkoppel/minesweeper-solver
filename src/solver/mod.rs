mod csp;
mod solutionset;
mod solve;

pub use std::collections::{HashSet, VecDeque};

pub use solve::*;
use crate::game::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Square {
    Empty,
    Mine,
    Num(usize),
}

pub use Square::*;

#[derive(Clone, Debug)]
pub struct Solver {
    grid: Vec<Vec<Square>>,
    active_squares: HashSet<Point>,
    game: Game
}

impl Solver {
    pub fn new(game: Game) -> Self {
        let (w, h) = game.size();

        Self {
            grid: vec![vec![Empty; w]; h],
            active_squares: HashSet::new(),
            game,
        }
    }

    fn get_point(&self, point: Point) -> &Square {
        &self.grid[point.1][point.0]
    }

    fn get_point_mut(&mut self, point: Point) -> &mut Square {
        &mut self.grid[point.1][point.0]
    }

    fn set_point(&mut self, point: Point, sq: Square) {
        self.grid[point.1][point.0] = sq;
    }

    pub fn uncover_point(&mut self, point: Point) {
        match self.game.explore_square(point) {
            Some(n) => self.set_point(point, Num(n)),
            None => {
                println!("{self}");
                self.set_point(point, Mine);
                println!("{self}");
                panic!("blew up at: {point:?}");
            }
        }
    }
}

use std::fmt;

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for x in row {
                match x {
                    Num(0) => write!(f, "``")?,
                    Num(n) => write!(f, "{n} ")?,
                    Mine => write!(f, "* ")?,
                    Empty => write!(f, "  ")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
