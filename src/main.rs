#![allow(dead_code)]

mod game;
mod bitvec;
mod solver;

use crate::game::*;
use crate::solver::*;

fn main() {
    // for _i in 0..10000 {
        let mut game = Game::new(MOORE_NEIGHBORHOOD.to_vec());
        // let mut game = Game::new(VON_NEUMANN_NEIGHBORHOOD.to_vec());

        // game.set_puzzle(vec![
            // vec![false, false, false, true , false],
            // vec![false, false, false, false, true ],
            // vec![false, false, true , false, false],
            // vec![false, false, false, false, false],
            // vec![false, false, true , false, false],
            // vec![false, false, false, false, true ],
            // vec![false, false, false, true , false],
            // vec![true , true , true , true , true ],
            // vec![false, false, false, false, false],
        // ]);

        // game.set_puzzle(vec![
            // vec![false, false, false, false, true ],
            // vec![false, false, false, false, false],
            // vec![false, false, false, true , false],
            // vec![false, false, true , true , false],
            // vec![true , false, false, false, false],
        // ]);

        let start = (0,0);
        game.random_puzzle((30,16), 99, start);

        // println!("{game}");

        let mut solver = Solver::new(game);
        solver.uncover_point(start);

        // solver.propogate(start);

        // let sols = solver.solve_csp(start);

        // for sol in &sols {
            // println!("{}", sol.len());
        // }

        // println!("vars: {}", sols.first().map(|s| s.variables()).unwrap_or(0));

    // }
}
