use std::{cell::RefCell, collections::HashMap, rc::Rc};

use by_address::ByAddress;
use itertools::Itertools;
use malachite::base::num::basic::traits::Zero;
use malachite::Natural;

use crate::game::*;
use crate::solution_set::solution_counting::natural_ratio_as_float;
use crate::solution_set::MineArrangements;
use crate::solver::Solver;
use crate::{bitset::BitSet, board::*};

struct Node<G: Graph> {
    total_solutions: Natural,
    safe_solutions: Natural,
    children: Vec<(Natural, usize)>,
    parents: HashMap<ByAddress<Rc<RefCell<Node<G>>>>, BitSet>,
    in_tree: bool,
}

pub struct Tree<G: Graph> {
    map: HashMap<Board<G>, Rc<RefCell<Node<G>>>>,
    root: Board<G>,
    solution_set: MineArrangements,
}

impl<G: Graph> Node<G> {
    fn new(arrangements: &MineArrangements) -> Self {
        let tile_safe = arrangements.tile_safe_solutions();
        let total_solutions = arrangements.total_solutions();
        let safe_solutions = tile_safe.iter().max().unwrap().clone();
        let children = tile_safe.into_iter().map(|tss| (tss, 0)).collect();

        Self {
            total_solutions,
            safe_solutions,
            children,
            parents: HashMap::new(),
            in_tree: true,
        }
    }

    fn add_parent(&mut self, pointer: &Rc<RefCell<Self>>, tile: usize) {
        let tiles = self
            .parents
            .entry(ByAddress(pointer.clone()))
            .or_insert_with(|| BitSet::empty(self.children.len()));

        if !tiles.get(tile) {
            tiles.set_to_one(tile);

            let mut node = pointer.borrow_mut();
            node.children[tile].0 -= &self.total_solutions - &self.safe_solutions;
            node.backpropogate();
        }
    }

    fn update(&mut self) -> Natural {
        let new_safe_solutions = self
            .children
            .iter()
            .map(|(tss, _)| tss)
            .max()
            .unwrap()
            .clone();

        let old_safe_solutions = std::mem::replace(&mut self.safe_solutions, new_safe_solutions);
        assert!(self.safe_solutions <= old_safe_solutions);

        old_safe_solutions - &self.safe_solutions
    }

    fn backpropogate(&mut self) {
        let loss = self.update();
        if loss == Natural::ZERO {
            return;
        }
        for (parent, tiles) in self.parents.iter() {
            let mut borrow = parent.borrow_mut();
            let children = &mut borrow.children;
            for tile in tiles.iter_ones() {
                children[tile].0 -= &loss;
            }
            std::mem::drop(borrow);
            parent.borrow_mut().backpropogate();
        }
    }

    fn move_to_search(&self) -> Option<usize> {
        let out = self
            .children
            .iter()
            .position_max_by_key(|(solutions, _)| solutions)?;

        (self.children[out].0 > Natural::ZERO).then_some(out)
    }

    fn best_move(&self) -> usize {
        self.children
            .iter()
            .position_max_by_key(|(tss, visits)| (visits, tss))
            .unwrap()
    }
}

impl<G: Graph> Tree<G> {
    pub fn new(root: Board<G>) -> Self {
        let mut map = HashMap::new();
        let solution_set = root.solutionset();
        map.insert(
            root.normalize(),
            Rc::new(RefCell::new(Node::new(&solution_set))),
        );
        Self {
            map,
            root,
            solution_set,
        }
    }

    pub fn set_root(&mut self, root: Board<G>) {
        self.map.retain(|board, node| {
            if board.subset_of(&root) {
                return true;
            }
            node.borrow_mut().in_tree = false;
            false
        });

        for node in self.map.values() {
            node.borrow_mut()
                .parents
                .retain(|parent, _| parent.borrow().in_tree);
        }
    }

    pub fn step(&mut self) {
        let mut board = self.root.clone();
        let mut node = self.node(&board).clone();
        let mut tile;

        // Make the best guess that we know of and solve until we reach a state we haven't reached
        // before or we finish the game.
        loop {
            if let Some(t) = node.borrow().move_to_search() {
                tile = t
            } else {
                return;
            }

            node.borrow_mut().children[tile].1 += 1;

            board.assert_tile(tile);
            let game = board.solutionset().sample_game_with_board(&board);
            board.clear_tile(tile);

            let mut solver = Solver::new(board, game);
            solver.uncover_tile(tile).unwrap();
            solver.solve().unwrap();

            board = solver.into_board();

            if board.is_solved() {
                return;
            }

            if let Some(next_node) = self.get_node(&board).cloned() {
                next_node.borrow_mut().add_parent(&node, tile);
                node = next_node;
            } else {
                break;
            }
        }

        // Create a new node
        let solutions = board.solutionset();
        let mut new_node = Node::new(&solutions);
        new_node.add_parent(&node, tile);

        let new_node = Rc::new(RefCell::new(new_node));
        self.map.insert(board.normalize(), new_node.clone());
    }

    fn get_node(&self, board: &Board<G>) -> Option<&Rc<RefCell<Node<G>>>> {
        self.map.get(&board.normalize())
    }

    fn node(&self, board: &Board<G>) -> &Rc<RefCell<Node<G>>> {
        &self.map[&board.normalize()]
    }

    fn root_node(&self) -> &Rc<RefCell<Node<G>>> {
        self.node(&self.root)
    }

    pub fn best_move(&self) -> usize {
        self.root_node().borrow().best_move()
    }

    pub fn prob_upper_bound(&self) -> Vec<f64> {
        let node = self.root_node().borrow();
        let total_solutions = &node.total_solutions;

        node.children
            .iter()
            .map(|(tss, _)| natural_ratio_as_float(tss, total_solutions))
            .collect()
    }

    pub fn visit_counts(&self) -> Vec<usize> {
        self.root_node()
            .borrow()
            .children
            .iter()
            .map(|(_, visits)| *visits)
            .collect()
    }
}

pub fn search_fn<G: Graph>(num_steps: usize) -> impl Fn(Board<G>) -> usize {
    move |board| {
        let mut tree = Tree::new(board);
        for _ in 0..num_steps {
            tree.step();
        }
        tree.best_move()
    }
}

impl<Gr: Graph, Ga: Game<Graph = Gr>> Solver<Gr, Ga>
where
    Board<Gr>: std::fmt::Display,
{
    pub fn solve_with_search(&mut self, search: impl Fn(Board<Gr>) -> usize) -> Option<()> {
        while !self.is_solved() {
            println!("{}", self.board());
            self.uncover_tile(search(self.board().clone()))?;
            self.solve()?;
        }

        Some(())
    }
}
