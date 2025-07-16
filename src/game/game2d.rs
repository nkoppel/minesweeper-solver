use super::*;
use std::{fmt, rc::Rc};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Graph2d {
    width: usize,
    height: usize,
    neighbors: Rc<[(isize, isize)]>,
}

impl Graph2d {
    pub fn new(width: usize, height: usize, neighbors: &[(isize, isize)]) -> Self {
        Self {
            width,
            height,
            neighbors: Rc::from(neighbors),
        }
    }
}

impl Graph for Graph2d {
    fn num_tiles(&self) -> usize {
        self.width * self.height
    }

    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> + '_ {
        let x = pos % self.width;
        let y = pos / self.width;

        self.neighbors.iter().filter_map(move |(xi, yi)| {
            let x2 = (x as isize + xi) as usize;
            let y2 = (y as isize + yi) as usize;

            (x2 < self.width && y2 < self.height).then(|| x2 + y2 * self.width)
        })
    }
}

impl fmt::Display for InternalGame<Graph2d> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(grid) = self.grid.as_ref() else {
            return write!(f, "None");
        };

        for (i, c) in grid.iter().take(self.graph.num_tiles()).enumerate() {
            if c {
                write!(f, "* ")?;
            } else {
                write!(f, ". ")?;
            }
            if i % self.graph.width == self.graph.width - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

use crate::board::*;

impl fmt::Display for Board<Graph2d> {
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
            if i % self.graph.width == self.graph.width - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
