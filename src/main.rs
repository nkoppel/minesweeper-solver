#![allow(dead_code)]

mod game;
mod nn;
mod search;
mod solver;

pub use crate::game::*;
pub use crate::nn::*;
pub use crate::solver::*;
pub use crate::search::*;

fn print_probs_2d(probs: &[f64], width: usize) {
    for (i, prob) in probs.iter().enumerate() {
        print!("{prob:5.3} ");

        if i % width == width - 1 {
            println!()
        }
    }
}

fn main() {
    // test();

    let game = InternalGame::new(10, StartType::Safe, Graph2d::new(9, 9, &MOORE_NEIGHBORHOOD));
    let eval = DummyEvalFunction;
    let mut searcher = Searcher::new(game, eval);

    for _ in 0..100000 { searcher.expand(); }

    println!("{}", searcher.best_action());
}
