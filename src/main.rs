#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![allow(dead_code)]

mod game;
mod search;

mod bitset;
mod board;
mod solution_set;
mod solver;

use solver::Solver;

use crate::board::Board;
pub use crate::game::*;
pub use crate::search::*;

pub(crate) fn print_probs_2d(probs: &[f64], width: usize) {
    for (i, prob) in probs.iter().enumerate() {
        if *prob == 1. {
            print!("## ");
        } else {
            print!("{:2} ", (prob * 100.).round());
        }

        if i % width == width - 1 {
            println!()
        }
    }
    println!()
}

pub(crate) fn print_counts_2d(counts: &[usize], width: usize) {
    for (i, count) in counts.iter().enumerate() {
        print!("{count:4} ");

        if i % width == width - 1 {
            println!()
        }
    }
    println!()
}

fn main() {
    let (width, height) = (16, 30);
    let num_mines = 99;
    let graph = Graph2d::new(width, height, &MOORE_NEIGHBORHOOD);
    let mut board = Board::new(graph.clone(), num_mines);
    // board.set_tile(0, 0).unwrap();
    // board.set_tile(1, 1).unwrap();
    // board.set_tile(width, 1).unwrap();
    // board.set_tile(width + 1, 2).unwrap();
    // board.set_tile(width * 2 + 2, 1).unwrap();

    // println!("{board}");
    // let solution_set = board.solutionset();
    // println!("{}", solution_set.total_solutions());
    // print_probs_2d(&solution_set.tile_safe_probability(), width);

    let mut tree = Tree::new(board.clone());
    for _ in 0..30000 {
        tree.step();
    }
    board.assert_tile(tree.best_move());
    println!("{}", tree.best_move());
    println!("{board}");

    // print_counts_2d(&tree.visit_counts(), width);
    // print_probs_2d(&tree.prob_upper_bound(), width);

    // let mut wins = 0.;
    // let mut games = 0.;

    // loop {
    // let mut solver = Solver::from_board(board.clone(), StartType::Safe);
    // solver.uncover_tile(0).unwrap();
    // solver.solve().unwrap();

    // let won = solver.solve_with_search(search_fn(10000)).is_some();

    // // let won = loop {
    // // if solver.is_solved() {
    // // break true;
    // // }
    // // println!("{}", solver.board());
    // // let mut tree = Tree::new(solver.board().clone());
    // // for _ in 0..10000 {
    // // tree.step();
    // // }
    // // if solver.uncover_tile(tree.best_move()).is_none() {
    // // break false;
    // // }
    // // if solver.solve().is_none() {
    // // break false;
    // // }
    // // };

    // games += 1.;
    // wins += won as u8 as f64;

    // println!("{games} {won} {}", wins / games);
    // }
}
