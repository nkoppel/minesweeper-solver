use crate::game::*;

pub use std::collections::{HashSet, HashMap};

mod solve;

pub use crate::solver::solve::*;

#[derive(Clone, Debug)]
pub struct Field {
    nmines: usize,
    points: HashSet<Point>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Square {
    Empty,
    Mine,
    Num(usize),
    Active(HashSet<usize>)
}

pub use Square::*;

impl Field {
    pub fn new(nmines: usize) -> Self {
        Self {
            nmines,
            points: HashSet::new(),
        }
    }

    pub fn solved_status(&self) -> Option<bool> {
        if  self.nmines == 0 ||
            self.points.is_empty() ||
            self.nmines == self.points.len()
        {
            Some(true)
        } else if self.nmines == 0 {
            Some(false)
        } else {
            None
        }
    }
}

use std::borrow::Borrow;
use std::mem;

#[derive(Clone, Debug)]
pub struct Solver {
    grid: Vec<Vec<Square>>,
    fields: HashMap<usize, Field>,
    field_id: usize,
    game: Game
}

impl Solver {
    pub fn new(game: Game) -> Self {
        let (w, h) = game.size();

        Self {
            grid: vec![vec![Empty; w]; h],
            fields: HashMap::new(),
            field_id: 0,
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
        match self.game.get_square(point) {
            Some(n) => self.set_point(point, Num(n)),
            None => panic!("Blew up")
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
                    Num(n) => write!(f, "{} ", n)?,
                    Mine => write!(f, "* ")?,
                    Empty | Active(_) => write!(f, "  ")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
