#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![allow(dead_code)]

mod game;
mod search;
// mod solver;
// mod solver2;
// mod solver3;

mod bitset;
mod board;
mod solution_set;
mod solver;

pub use crate::game::*;
pub use crate::search::*;
// pub use crate::solver::*;
// pub use crate::solver2::*;
// pub use crate::solver3::*;

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
    // let mut board = crate::board::Board::new(Graph2d::new(8, 8, &MOORE_NEIGHBORHOOD), 10);
    // board.set_tile(0, 0);
    // board.set_tile(1, 1);
    // board.set_tile(8, 1);
    // board.set_tile(9, 2);
    // board.set_tile(18, 1);

    // println!("{board}");
    // let solution_set = board.solutionset();
    // print_probs_2d(&solution_set.tile_safe_probability(), 8);

    // // let best_move = best_move(&board, 10000);
    // let best_move = board.best_move_safes().unwrap();
    // println!("{best_move}");

    // board.assert_tile(best_move);

    // println!("{board}");

    // println!("{:?}", board.solutionset().total_solutions());
    // println!("{:?}", board.solutionset().total_solutions());
    // board.set_tile(27, 1);
    // println!("{:?}", board.solutionset().total_solutions());

    // let solution_set = board.solutionset();

    // board.set_tile(18, 1);

    // let solution_set_2 = solution_set.increment(&board);

    // println!("{board}");

    // print_probs_2d(&solution_set.tile_safe_probability(), 8);
    // print_probs_2d(&solution_set_2.tile_safe_probability(), 8);

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

        let mut solver = solver::Solver::from_game(game);
        solver.uncover_tile(0).unwrap();
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
