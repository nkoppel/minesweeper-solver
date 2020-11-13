use rand::{thread_rng, Rng};

use std::collections::HashSet;

pub type Point = (usize, usize);
pub type RelPoint = (isize, isize);

#[derive(Clone, Debug)]
pub struct Game {
    pub grid: Vec<Vec<bool>>,
    pub nmines: usize,
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

fn valid_neighbors(neighbors: &Vec<RelPoint>, w: usize, h: usize, x: usize, y: usize)
    -> Vec<Point>
{
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

fn gen_double_neighbors(neighbors: &Vec<RelPoint>) -> Vec<RelPoint> {
    let mut out = HashSet::new();

    for p1 in neighbors {
        for p2 in neighbors {
            out.insert((p1.0 + p2.0, p1.1 + p2.1));
        }
    }

    for p in neighbors {
        out.remove(p);
    }
    out.remove(&(0,0));

    out.into_iter().collect()
}

impl Game {
    pub fn new(neighbors: Vec<RelPoint>) -> Self {
        let double_neighbors = gen_double_neighbors(&neighbors);

        Game{grid: Vec::new(), nmines: 0, neighbors, double_neighbors}
    }

    pub fn random_puzzle(&mut self, width: usize, height: usize, mines: usize, start: Point) {
        let (x, y) = start;
        let mut rng = thread_rng();
        let squares = width * height;

        let mut no_mines = valid_neighbors(&self.neighbors, width, height, x, y);
        no_mines.push((x, y));

        let mut random_mines = vec![0; squares - no_mines.len()];
        let mut j = 0;

        self.grid = vec![vec![false; width]; height];

        for i in 0..random_mines.len() {
            let x2 = i % width;
            let y2 = i / width;

            if !no_mines.contains(&(x2, y2)) {
                random_mines[j] = i;
                j += 1;
            }
        }

        for i in 0..mines {
            random_mines.swap(i, rng.gen_range(i, squares - no_mines.len()));
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
        let (x, y) = point;
        let (w, h) = self.size();

        valid_neighbors(&self.neighbors, w, h, x, y)
    }

    pub fn get_double_neighbors(&self, point: Point) -> Vec<Point> {
        let (x, y) = point;
        let (w, h) = self.size();

        valid_neighbors(&self.double_neighbors, w, h, x, y)
    }

    pub fn get_square(&mut self, point: Point) -> Option<usize> {
        let (x, y) = point;

        if self.grid[y][x] {
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
