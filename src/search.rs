use itertools::Itertools;
use malachite::{num::basic::traits::Zero, Natural};

use crate::board::*;
use crate::game::*;
use crate::solution_set::solution_counting::natural_ratio_as_float;

#[derive(Clone, Debug)]
struct Node {
    total_solutions: Natural,
    safe_solutions: Natural,
    max_child: Natural,
    visits: usize,
    children: Vec<Result<Vec<Node>, Natural>>,
}

impl Node {
    fn new(board: &Board<impl Graph>) -> Self {
        let solutionset = board.solutionset();
        let tile_safe_solutions = solutionset.tile_safe_solutions();
        let max_child = tile_safe_solutions.iter().max().unwrap().clone();

        Node {
            total_solutions: solutionset.total_solutions(),
            safe_solutions: max_child.clone(),
            max_child,
            visits: 1,
            children: tile_safe_solutions.into_iter().map(Err).collect(),
        }
    }

    fn update(&mut self) {
        self.safe_solutions = self
            .children
            .iter()
            .map(|tile| match tile {
                Ok(children) => children.iter().map(|child| &child.safe_solutions).sum(),
                Err(count) => count.clone(),
            })
            .sum();

        self.max_child = self
            .children
            .iter()
            .map(|tile| match tile {
                Ok(children) => children
                    .iter()
                    .map(|child| &child.max_child)
                    .max()
                    .cloned()
                    .unwrap(),
                Err(count) => count.clone(),
            })
            .max()
            .unwrap();

        self.visits += 1;
    }

    fn expand(&mut self, mut board: Board<impl Graph>) {
        if self.total_solutions == Natural::ZERO {
            return;
        }

        let tile = (0..self.children.len())
            .max_by_key(|&tile| match &self.children[tile] {
                Ok(children) => children.iter().map(|child| &child.safe_solutions).sum(),
                Err(count) => count.clone(),
            })
            .unwrap();

        if let Ok(children) = &mut self.children[tile] {
            let hint = children
                .iter()
                .position_max_by_key(|child| &child.max_child)
                .unwrap();
            board.set_tile(tile, hint as u8);
            children[hint].expand(board);
        } else {
            let children = (0..=board.neighbors(tile).count())
                .map(|hint| {
                    board.set_tile(tile, hint as u8);
                    Node::new(&board)
                })
                .collect();

            self.children[tile] = Ok(children)
        }

        self.update()
    }

    fn best_move(&self) -> usize {
        self.children
            .iter()
            .position_max_by_key(|tile| match tile {
                Ok(children) => children.iter().map(|child| child.visits).sum(),
                Err(_) => 0,
            })
            .unwrap()
    }
}

pub fn best_move(board: &Board<impl Graph>, steps: usize) -> usize {
    let mut tree = Node::new(board);

    for _ in 0..steps {
        tree.expand(board.clone());
    }

    // println!("{:?}",
    // tree.children[27].as_ref().unwrap().iter().map(|x| &x.safe_solutions).collect_vec());
    // println!("{:?}",
    // tree.children[27].as_ref().unwrap().iter().map(|x| &x.safe_solutions).sum::<Natural>());
    println!(
        "{:?}",
        tree.children
            .iter()
            .map(|tile| match tile {
                Ok(children) => children.iter().map(|child| &child.visits).sum(),
                Err(_) => 0,
            })
            .collect_vec()
    );

    tree.best_move()
}

#[derive(PartialEq)]
struct OrderedF64(f64);

impl Eq for OrderedF64 {}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl<G: Graph> Board<G> {
    pub fn best_move_perplexity(&self) -> Option<usize> {
        let total_solutions = self.solutionset().total_solutions();

        (0..self.num_tiles())
            .filter(|i| self.grid[*i] == Empty)
            .max_by_key(|&i| {
                let mut board = self.clone();
                let mut total_safe_solutions = Natural::from(0u32);
                let mut solutions = Vec::new();

                for hint in 0..board.neighbors(i).count() {
                    board.set_tile(i, hint as u8);

                    let count = board.solutionset().total_solutions();
                    total_safe_solutions += &count;
                    solutions.push(count);
                }

                let perplexity: f64 = solutions
                    .iter()
                    .map(|sol| {
                        let p = natural_ratio_as_float(sol, &total_safe_solutions);
                        p.powf(-p)
                    })
                    .product();

                println!(
                    "{}",
                    perplexity * natural_ratio_as_float(&total_safe_solutions, &total_solutions)
                );

                OrderedF64(
                    perplexity * natural_ratio_as_float(&total_safe_solutions, &total_solutions),
                )
            })
    }
}

impl<G: Graph> Board<G> {
    pub fn best_move_safes(&self) -> Option<usize> {
        let total_solutions = self.solutionset().total_solutions();

        (0..self.num_tiles())
            .filter(|i| self.grid[*i] == Empty)
            .max_by_key(|&i| {
                let mut board = self.clone();

                let out = (0..board.neighbors(i).count())
                    .map(|hint| {
                        board.set_tile(i, hint as u8);

                        let solutionset = board.solutionset();
                        let has_safe = solutionset.solved().0.any();
                        solutionset.total_solutions() * Natural::from(has_safe as usize)
                    })
                    .sum::<Natural>();

                println!("{}", natural_ratio_as_float(&out, &total_solutions));

                out
            })
    }
}
