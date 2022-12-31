use rand::{thread_rng, Rng};

use std::collections::HashSet;

pub type Point = (usize, usize);
pub type RelPoint = (isize, isize);

#[derive(Clone, Debug)]
pub struct Game {
    pub grid: Vec<Vec<bool>>,
    pub nmines: usize,
    pub failed: bool,
    neighbors: Vec<RelPoint>,
    double_neighbors: Vec<RelPoint>,
}

pub const MOORE_NEIGHBORHOOD: [RelPoint; 8] = 
    [
        (-1, -1), ( 0, -1), ( 1, -1),
        (-1,  0),           ( 1,  0),
        (-1,  1), ( 0,  1), ( 1,  1)
    ];

pub const KNIGHT_NEIGHBORHOOD: [RelPoint; 8] =
    [
               (-1, -2),   ( 1, -2),
        (-2, -1),                 ( 2, -1),

        (-2,  1),                 ( 2,  1),
               (-1,  2),   ( 1,  2),
    ];

fn valid_neighbors(neighbors: &[RelPoint], size: Point, point: Point)
    -> Vec<Point>
{
    let (w, h) = size;
    let (x, y) = point;
    let mut out = Vec::new();

    for (xi, yi) in neighbors {
        let x2 = x as isize + *xi;
        let y2 = y as isize + *yi;

        if x2 >= 0 && y2 >= 0 && x2 < w as isize && y2 < h as isize {
            out.push((x2 as usize, y2 as usize));
        }
    }

    out
}

fn gen_double_neighbors(neighbors: &[RelPoint]) -> Vec<RelPoint> {
    let mut out = neighbors.iter().copied().collect::<HashSet<_>>();

    for (x1, y1) in neighbors {
        for (x2, y2) in neighbors {
            out.insert((x1 + x2, y1 + y2));
        }
    }

    let mut out = out.into_iter().collect::<Vec<_>>();
    out.sort_unstable();
    out
}

impl Game {
    pub fn new(neighbors: Vec<RelPoint>) -> Self {
        let double_neighbors = gen_double_neighbors(&neighbors);

        Game{grid: Vec::new(), failed: false, nmines: 0, neighbors, double_neighbors}
    }

    pub fn random_puzzle(&mut self, size: Point, mines: usize, start: Point) {
        let (width, height) = size;
        let mut rng = thread_rng();
        let squares = width * height;

        let mut no_mines = valid_neighbors(&self.neighbors, size, start);
        no_mines.push(start);

        let mut random_mines = vec![0; squares - no_mines.len()];
        let mut j = 0;

        self.grid = vec![vec![false; width]; height];

        for i in 0..width * height {
            let x2 = i % width;
            let y2 = i / width;

            if !no_mines.contains(&(x2, y2)) {
                random_mines[j] = i;
                j += 1;
            }
        }

        for i in 0..mines {
            random_mines.swap(i, rng.gen_range(i..squares - no_mines.len()));
            self.grid[random_mines[i] / width][random_mines[i] % width] = true;
        }
    }

    pub fn set_puzzle(&mut self, grid: Vec<Vec<bool>>) {
        self.grid = grid;
    }

    pub fn size(&self) -> (usize, usize) {
        (self.grid[0].len(), self.grid.len())
    }

    pub fn get_neighbors(&self, point: Point) -> Vec<Point> {
        valid_neighbors(&self.neighbors, self.size(), point)
    }

    pub fn get_double_neighbors(&self, point: Point) -> Vec<Point> {
        valid_neighbors(&self.double_neighbors, self.size(), point)
    }

    pub fn explore_square(&mut self, point: Point) -> Option<usize> {
        let (x, y) = point;

        if self.grid[y][x] {
            self.failed = true;
            return None;
        }

        let mut out = 0;

        for (x2, y2) in self.get_neighbors(point) {
            out += self.grid[y2][x2] as usize;
        }

        Some(out)
    }
}

use std::fmt;

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for x in row {
                if *x {
                    write!(f, "1 ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
