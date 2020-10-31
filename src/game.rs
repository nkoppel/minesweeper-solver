use rand::{thread_rng, Rng};

#[derive(Clone, Debug)]
pub struct Game {
    pub grid: Vec<Vec<bool>>,
    pub nmines: usize,
}

pub const NEIGHBORHOOD: [(isize, isize); 8] = 
    [
        (-1, -1), ( 0, -1), ( 1, -1),
        (-1,  0),           ( 1,  0),
        (-1,  1), ( 0,  1), ( 1,  1)
    ];

fn get_neighbors(w: usize, h: usize, x: usize, y: usize)
    -> Vec<(usize, usize)>
{
    let mut out = Vec::new();

    for (xi, yi) in &NEIGHBORHOOD {
        let x2 = x as isize + *xi;
        let y2 = y as isize + *yi;

        if x2 >= 0 && y2 >= 0 && x2 < w as isize && y2 < h as isize {
            out.push((x2 as usize, y2 as usize));
        }
    }

    out
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize, x: usize, y: usize)
        -> Self
    {
        let mut rng = thread_rng();
        let squares = width * height;

        let mut neighbors = get_neighbors(width, height, x, y);
        neighbors.push((x, y));

        let mut random_mines = vec![0; squares - neighbors.len()];
        let mut grid = vec![vec![false; width]; height];
        let mut j = 0;

        for i in 0..random_mines.len() {
            let x2 = i % width;
            let y2 = i / width;

            if !neighbors.contains(&(x2, y2)) {
                random_mines[j] = i;
                j += 1;
            }
        }

        for i in 0..mines {
            random_mines.swap(i, rng.gen_range(i, squares - neighbors.len()));
            grid[random_mines[i] / width][random_mines[i] % width] = true;
        }

        Game{grid, nmines: mines}
    }

    pub fn size(&self) -> (usize, usize) {
        (self.grid[0].len(), self.grid.len())
    }

    pub fn get_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let (w, h) = self.size();

        get_neighbors(w, h, x, y)
    }

    pub fn get_square(&mut self, x: usize, y: usize) -> Option<usize> {
        if self.grid[y][x] {
            return None;
        }

        let mut out = 0;

        for (x2, y2) in self.get_neighbors(x, y) {
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
