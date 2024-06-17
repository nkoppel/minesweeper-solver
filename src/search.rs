use std::collections::HashMap;

use ibig::{ubig, UBig};
use itertools::Itertools;

use crate::game::*;
use crate::solver::*;

struct TreeNode {
    tile_win_count: Vec<UBig>,
    tile_visit_count: Vec<usize>,
    child_win_counts: Vec<HashMap<Vec<Tile>, UBig>>,
    total_solution_count: UBig,
}

pub struct Tree<G: Graph> {
    nodes: HashMap<Vec<Tile>, TreeNode>,
    root: Board<G>,
}

impl TreeNode {
    pub fn new(board: &Board<impl Graph>) -> Self {
        Self::new_with_solution_set(board, &board.get_solutionset())
    }

    pub fn new_with_solution_set(board: &Board<impl Graph>, solution_set: &SolutionSet) -> Self {
        Self {
            tile_win_count: solution_set.tile_safe_counts(),
            tile_visit_count: vec![0; board.num_tiles()],
            child_win_counts: vec![HashMap::new(); board.num_tiles()],
            total_solution_count: solution_set.total_solution_count(),
        }
    }
}

impl<G: Graph> Tree<G> {
    pub fn new(root: Board<G>) -> Self {
        Self {
            nodes: HashMap::new(),
            root,
        }
    }

    pub fn set_root(&mut self, root: Board<G>) {
        self.root = root;

        self.nodes
            .retain(|k, _| is_grid_subset_of(k, &self.root.grid))
    }

    #[allow(clippy::assigning_clones)]
    pub fn expand(&mut self) {
        let mut rng = rand::thread_rng();

        let mut stack: Vec<Board<G>> = Vec::new();
        let mut board = self.root.clone();
        let mut solution_set: Option<SolutionSet> = None;

        while let Some(node) = self.nodes.get(&board.grid) {
            let best_tile = node.tile_win_count.iter().position_max().unwrap();

            if board.grid[best_tile] != Empty {
                return;
            }
            board.assert_tile(best_tile);
            let sample = board.get_solutionset().sample_game(&mut rng);
            board.clear_tile(best_tile);

            let mut board2 = board.clone();
            let mut game = InternalGame::from_grid(sample, board.graph.clone());
            let mut solver = Solver::new(&mut board2, &mut game);

            solver.uncover_tile(best_tile).unwrap();
            solver.propogate(&mut vec![best_tile]);
            solution_set = Some(solver.solve_csp());

            stack.push(board);
            board = board2;
        }

        let solution_set = solution_set.unwrap_or_else(|| board.get_solutionset());
        let node = TreeNode::new_with_solution_set(&board, &solution_set);

        let mut win_count = node
            .tile_win_count
            .iter()
            .max()
            .unwrap()
            .clone()
            .max(ubig!(1));
        let mut total_solution_count = node.total_solution_count.clone();

        self.nodes.insert(board.grid.clone(), node);
        stack.push(board);

        for boards in stack.windows(2).rev() {
            let board1 = &boards[0];
            let board2 = &boards[1];

            let node = self.nodes.get_mut(&board1.grid).unwrap();
            let best_tile = node.tile_win_count.iter().position_max().unwrap();

            node.tile_visit_count[best_tile] += 1;
            let prev_win_count = node.child_win_counts[best_tile]
                .entry(board2.grid.clone())
                .or_insert_with(|| total_solution_count.clone());
            node.tile_win_count[best_tile] -= &*prev_win_count - win_count.clone();
            *prev_win_count = win_count;

            win_count = node.tile_win_count.iter().max().unwrap().clone();
            total_solution_count.clone_from(&node.total_solution_count);
        }
    }

    pub fn best_guess(&self) -> usize {
        let node = &self.nodes[&self.root.grid];
        println!("{:?}", node.tile_win_count);
        println!("{:?}", node.tile_win_count.iter().position_max());
        println!("{:?}", node.tile_visit_count);
        self.nodes[&self.root.grid]
            .tile_visit_count
            .iter()
            .position_max()
            .unwrap()
    }
}

const EXPLORATION_FACTOR: f64 = 1.0;

struct MCTSNode {
    tile_visit_count: Vec<usize>,
    tile_win_count: Vec<f64>,
    tile_safe_probability: Vec<f64>,
}

pub struct MCTSTree<G: Graph> {
    nodes: HashMap<Vec<Tile>, MCTSNode>,
    root: Board<G>,
}

impl MCTSNode {
    fn new(board: &Board<impl Graph>) -> Self {
        Self::new_with_solution_set(board, &board.get_solutionset())
    }

    fn new_with_solution_set(board: &Board<impl Graph>, solution_set: &SolutionSet) -> Self {
        Self {
            tile_visit_count: vec![0; board.num_tiles()],
            tile_win_count: vec![0.; board.num_tiles()],
            tile_safe_probability: solution_set.tile_safe_probabilities(),
        }
    }

    fn next_search(&self) -> Option<usize> {
        let total_visits = self.tile_visit_count.iter().sum::<usize>().max(1);
        let ln_total_visits = (total_visits as f64).ln();

        self.tile_visit_count
            .iter()
            .zip(&self.tile_win_count)
            .zip(&self.tile_safe_probability)
            .map(|((&visit_count, &win_count), &safe_probability)| {
                if safe_probability == 0. {
                    return 0.;
                }

                let win_prob = if visit_count > 0 {
                    win_count / visit_count as f64
                } else {
                    safe_probability
                };

                let exploration = (ln_total_visits / (visit_count as f64 + 1.)).sqrt();
                win_prob + EXPLORATION_FACTOR * exploration
            })
            .position_max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn update(&mut self, action: usize, win_prob: f64) -> f64 {
        self.tile_visit_count[action] += 1;
        self.tile_win_count[action] += win_prob;

        self.tile_safe_probability[action] * win_prob
    }
}

impl<G: Graph> MCTSTree<G> {
    pub fn new(root: Board<G>) -> Self {
        Self {
            nodes: HashMap::new(),
            root,
        }
    }

    pub fn set_root(&mut self, root: Board<G>) {
        self.root = root;
    }

    pub fn prune(&mut self) {
        self.nodes
            .retain(|k, _| is_grid_subset_of(k, &self.root.grid))
    }

    pub fn expand(&mut self) {
        let mut rng = rand::thread_rng();

        let mut stack: Vec<(Board<G>, usize)> = Vec::new();
        let mut board = self.root.clone();

        while !board.is_solved() {
            let node = self
                .nodes
                .entry(board.grid.clone())
                .or_insert_with(|| MCTSNode::new(&board));
            let next_search = node.next_search().unwrap();

            let mut board2 = board.clone();

            board2.assert_tile(next_search);
            let mut game = InternalGame::from_grid(
                board2.get_solutionset().sample_game(&mut rng),
                board.graph.clone(),
            );
            board2.clear_tile(next_search);

            let mut solver = Solver::new(&mut board2, &mut game);

            solver.uncover_tile(next_search).unwrap();
            solver.propogate(&mut vec![next_search]);
            solver.solve_csp();

            stack.push((board, next_search));
            board = board2;
        }

        let mut win_prob = 1.;

        for (board, action) in stack.into_iter().rev() {
            let node = self.nodes.get_mut(&board.grid).unwrap();
            win_prob = node.update(action, win_prob);
        }
    }

    pub fn best_guess(&self) -> usize {
        let node = &self.nodes[&self.root.grid];
        println!("{:?}", node.tile_win_count);
        println!("{:?}", node.tile_visit_count);
        return self.nodes[&self.root.grid]
            .tile_visit_count
            .iter()
            .position_max()
            .unwrap();
    }
}
