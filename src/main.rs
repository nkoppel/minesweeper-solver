mod game;
mod solver;

use crate::game::*;
use crate::solver::*;

fn main() {
    let mut game = Game::new(MOORE_NEIGHBORHOOD.to_vec());

    game.set_puzzle(vec![
        vec![true , true , false, false, true ],
        vec![false, false, false, false, true ],
        vec![false, false, false, false, true ],
        vec![false, false, true , false, true ],
        vec![true , true , false, false, true ],
    ]);

    let mut solver = Solver::new(game);
    solver.uncover_point((0, 2));
    solver.propogate((0, 2));

    println!("{}", solver);
}
