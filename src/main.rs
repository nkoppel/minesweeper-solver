#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![allow(dead_code)]

mod game;
mod search;
mod solver;
mod solver2;
mod solver3;

mod bitset;
mod board;
mod padded_zip;
mod solution_set;
mod solver_struct;

pub use crate::game::*;
// pub use crate::search::*;
pub use crate::solver::*;
// pub use crate::solver2::*;
pub use crate::solver3::*;

pub(crate) fn print_probs_2d(probs: &[f64], width: usize) {
    for (i, prob) in probs.iter().enumerate() {
        print!("{prob:5.3} ");

        if i % width == width - 1 {
            println!()
        }
    }
}

// fn generate_game() -> InternalGame<Graph2d> {
// InternalGame::new(
// 10,
// StartType::SafeNeighborhood,
// Graph2d::new(8, 8, &MOORE_NEIGHBORHOOD),
// )
// }

fn generate_game() -> InternalGame<Graph2d> {
    InternalGame::new(
        99,
        StartType::SafeNeighborhood,
        Graph2d::new(30, 16, &MOORE_NEIGHBORHOOD),
    )
}

fn main() {
    // let mut game = InternalGame::from_grid(bitvec::bitvec![usize, bitvec::order::Lsb0; 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1], Graph2d::new(4, 4, &MOORE_NEIGHBORHOOD));

    for i in 0..100000 {
        println!("{i}");
        let mut game = generate_game();
        // let game = InternalGame::from_grid(
        // bitvec::bitvec![usize, bitvec::order::Lsb0;
        // 0,0,0,0,0,0,1,1,
        // 0,0,0,0,0,0,0,1,
        // 0,1,0,0,0,1,0,0,
        // 0,1,0,0,0,0,0,0,
        // 0,0,0,0,0,0,1,0,
        // 0,0,1,0,0,0,1,0,
        // 0,0,0,0,0,0,0,0,
        // 1,0,0,0,0,0,0,0,
        // ],
        // Graph2d::new(8, 8, &MOORE_NEIGHBORHOOD),
        // );

        // let mut solver = MineArrangements::from_game(&game);
        // solver.add_constraint_with_game(0, &mut game).unwrap();
        // println!("{game}");
        // solver.play_game(&mut game);

        // let mut board = Board::from_game(&game);
        // let mut solver = Solver::new(&mut board, &mut game);
        // solver.uncover_tile(0).unwrap();
        // solver.propogate(&mut vec![0]);
        // solver.solve_csp();

        let mut solver = solver_struct::Solver::from_game(game);
        solver.uncover_tile(0).unwrap();
        solver.propogate(&mut vec![0]).unwrap();
        solver.solve().unwrap();
    }

    // println!("{:x?}", solver);

    // let mut board = Board::from_game(&game);

    // let mut tree = MCTSTree::new(board.clone());
    // let mut tree = Tree::new(board.clone());

    // for _ in 0..10000 {
    // tree.expand();
    // }

    // println!("{}", tree.best_guess());
    // println!("{}", board);
    // board.assert_tile(tree.best_guess());
    // println!("{}", board);

    // let mut wins: f64 = 0.;
    // let mut games: f64 = 0.;

    // loop {
    // let mut game = generate_game();
    // let mut board = Board::from_game(&game);

    // let mut tree = MCTSTree::new(board.clone());
    // let mut solver = Solver::new(&mut board, &mut game);

    // let result = loop {
    // for _ in 0..1000 {
    // tree.expand();
    // }

    // let Some(()) = solver.uncover_tile(tree.best_guess()) else {
    // break 0.;
    // };

    // solver.solve_csp();

    // if solver.board().remaining_empty_tiles() == 0 {
    // break 1.;
    // }

    // tree.set_root(solver.board().clone());
    // tree.prune();
    // };

    // wins += result;
    // games += 1.;

    // println!("{}", wins / games);
    // }
}
