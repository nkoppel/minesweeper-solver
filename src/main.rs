#![allow(dead_code)]

mod game;
mod solver;
mod bitvec;
mod csp;

use crate::game::*;
use crate::solver::*;
use crate::csp::*;

fn main() {
    // game.set_puzzle(vec![
        // vec![true , true , false, false, true ],
        // vec![false, false, false, false, true ],
        // vec![false, false, false, false, true ],
        // vec![false, false, true , false, true ],
        // vec![true , true , false, false, true ],
    // ]);

    // game.set_puzzle(vec![
        // vec![false, false, false],
        // vec![false, false, false],
        // vec![true , false, true ],
    // ]);
    // let start = (0,0);

    // game.set_puzzle(vec![
        // vec![false, false, true , true , false, false],
        // vec![false, false, false, false, false, false],
        // vec![true , false, false, false, false, true ],
        // vec![true , false, false, false, false, true ],
        // vec![false, false, false, false, false, false],
        // vec![false, false, true , true , false, false],
    // ]);
    // let start = (2,2);

    // game.set_puzzle(vec![
        // vec![false, false, false, true , false, false],
        // vec![false, false, false, false, false, false],
        // vec![false, false, true , false, false, false],
        // vec![true , false, false, false, false, false],
        // vec![false, false, false, false, false, false],
        // vec![false, false, false, false, false, false],
    // ]);
    // let start = (0,0);

    // game.set_puzzle(vec![
        // vec![false, false, false, false, false, false, false],
        // vec![false, false, false, false, false, false, true ],
        // vec![false, false, true , false, false, false, false],
        // vec![false, true , true , false, false, false, false],
        // vec![false, false, false, false, true , true , false],
        // vec![false, false, false, false, false, false, false],
    // ]);
    // let start = (0,0);

    for i in 0..10000 {
        let mut game = Game::new(MOORE_NEIGHBORHOOD.to_vec());
        let start = (0,0);

        // game.set_puzzle(vec![
            // vec![false, false, false, false, false, false, false, false, false, false],
            // vec![false, false, false, false, false, false, false, false, true , false],
            // vec![false, false, false, false, false, false, true , false, false, false],
            // vec![false, false, false, false, false, false, false, false, false, false],
            // vec![false, false, true , true , true , false, true , false, true , false],
            // vec![false, false, true , true , false, true , false, false, false, false],
            // vec![false, false, true , false, true , true , false, true , false, true ],
            // vec![false, false, false, false, false, false, false, false, true , true ],
            // vec![false, false, false, false, false, false, false, false, false, false],
            // vec![false, false, false, false, false, false, false, false, false, false],
        // ]);

        game.random_puzzle((30,16), 99, start);

        // println!("{game}");

        let mut solver = Solver::new(game);
        solver.uncover_point(start);
        solver.propogate(start);

        // println!("{}", solver);

        solver.solve_csp();

        // println!("{}", solver);
    }
}
