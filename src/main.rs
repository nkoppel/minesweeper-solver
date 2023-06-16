#![allow(dead_code)]

mod game;
mod search;
mod solver;

pub use crate::game::*;
pub use crate::solver::*;
use rand::prelude::*;

fn print_probs_2d(probs: &[f64], width: usize) {
    for (i, prob) in probs.iter().enumerate() {
        print!("{prob:5.3} ");

        if i % width == width - 1 {
            println!()
        }
    }
}

fn main() {
    // let game = Game2d::from_2d_grid(
    // MOORE_NEIGHBORHOOD.to_vec(),
    // &[
    // vec![false, false, false, false, false, false, false, false],
    // vec![false, false, false, false, false, false, false, false],
    // vec![true , false, false, true , true , false, false, true ],
    // vec![false, true , false, false, false, true , false, false],
    // ],
    // );
    // let start = 0;

    // for _ in 0..100000 {
    let game = SafeStartGame::new(Game2d::new(
        16,
        30,
        99,
        MOORE_NEIGHBORHOOD.to_vec(),
        &mut thread_rng(),
    ));
    let start = 0;

    // println!("{game}");

    let mut solver = Solver::new(game);
    solver.uncover_square(start);
    solver.propogate(&mut vec![start]);

    println!("{solver}");

    let sols = solver.solve_csp().unwrap();

    print_probs_2d(&sols.square_mine_probabilities(), 16);

    println!("{solver}");
    // println!("{sols:?}");
    // println!("{:?}", sols.1.iter().map(|s| s.num_solutions()).collect::<Vec<_>>());

    // println!("{solver}");
    // }
}
