#![allow(dead_code)]

mod game;
mod solver;
// mod search;

pub use crate::game::*;
pub use crate::solver::*;

fn main() {
    // let mut game = Game2d::from_2d_grid(
    // MOORE_NEIGHBORHOOD.to_vec(),
    // &[
    // vec![false, false, true , true , false, false],
    // vec![false, false, false, false, false, false],
    // vec![true , false, false, false, false, true ],
    // vec![true , false, false, false, false, true ],
    // vec![false, false, false, false, false, false],
    // vec![false, false, true , true , false, false],
    // ],
    // );

    // let start = 14;

    // let mut game = Game2d::from_2d_grid(
        // MOORE_NEIGHBORHOOD.to_vec(),
        // &[
            // vec![false, false, false, false, false],
            // vec![false, false, false, false, false],
            // vec![false, false, true, false, false],
            // vec![true, false, false, true, false],
        // ],
    // );

    // let start = 0;
    let mut sum = 0;

    for _ in 0..100000 {
        let game = Game2d::new(30, 16, 99, MOORE_NEIGHBORHOOD.to_vec());
        let start = 0;

        // println!("{game}");

        let mut solver = Solver::new(game);
        solver.uncover_square(start);
        solver.propogate(&mut vec![start]);

        // println!("{solver}");

        let sols = solver.solve_csp();
        sum += sols.1.len();
        // println!("{sols:?}");
        // println!("{:?}", sols.1.iter().map(|s| s.num_solutions()).collect::<Vec<_>>());

        // println!("{solver}");
    }

    println!("{sum}");
}
