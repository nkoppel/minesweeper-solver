use std::collections::HashMap;

use itertools::Itertools;

use crate::game::*;
use crate::solver::*;

const EXPLORATION_FACTOR: f64 = 1.5;

struct TreeNode {
    policy: Vec<f64>,
    mine_prob: Vec<f64>,
    action_visits: Vec<usize>,
    action_values: Vec<f64>,
}

pub struct Searcher<E: EvalFunction, G: Graph> {
    tree: HashMap<Vec<Tile>, TreeNode>,
    root: Solver<InternalGame<G>>,
    eval: E,
}

use std::ops::Deref;

pub trait EvalFunction {
    /// Yields q values and policies
    fn eval_batch(
        &self,
        features: &[impl Deref<Target = [f64]>],
        masks: &[impl Deref<Target = [f64]>],
    ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>);

    fn train_batch(
        &mut self,
        features: &[impl Deref<Target = [f64]>],
        masks: &[impl Deref<Target = [f64]>],
        values: &[impl Deref<Target = [f64]>],
        policies: &[impl Deref<Target = [f64]>],
    );

    /// Yields q values and a policy
    fn eval(&self, features: &[f64], mask: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let (mut v, mut p) = self.eval_batch(&[features], &[mask]);
        (v.swap_remove(0), p.swap_remove(0))
    }
}

fn grid_as_mask(grid: &[Tile]) -> Vec<f64> {
    grid.iter().map(|&t| (t == Empty) as u8 as f64).collect()
}

impl SolutionSet {
    fn nn_features(&self) -> Vec<f64> {
        let mut out = self.tile_mine_probabilities();
        out.extend(self.grid.iter().map(|&cell| (cell == Empty) as u8 as f64));
        out.extend((0..self.grid.len()).map(|_| 1.0));
        out.extend(self.grid.iter().map(|&cell| match cell {
            Hint {
                remaining_mines, ..
            } => remaining_mines as f64 + 1.0,
            _ => 0.0,
        }));

        assert_eq!(out.len(), self.grid.len() * 4);

        out
    }

    fn make_treenode(&self, eval: &impl EvalFunction) -> TreeNode {
        let num_tiles = self.grid.len();

        let features = self.nn_features();
        let mine_prob = features[..num_tiles].to_vec();
        let mask: Vec<f64> = grid_as_mask(&self.grid);
        let (mut action_values, policy) = eval.eval(&features, &mask);

        for (val, prob) in action_values.iter_mut().zip(mine_prob.iter()) {
            *val *= 1. - *prob;
        }

        TreeNode {
            policy,
            mine_prob,
            action_visits: mask.into_iter().map(|x| x as usize).collect(),
            action_values,
        }
    }
}

impl TreeNode {
    fn next_search_action(&self) -> usize {
        let total_visits = self.action_visits.iter().sum::<usize>() as f64;

        (0..self.policy.len())
            .filter(|i| self.policy[*i] > 0.)
            .map(|i| {
                self.action_values[i]
                    + EXPLORATION_FACTOR * self.policy[i] * total_visits.sqrt()
                        / (1 + self.action_visits[i]) as f64
            })
            .position_max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    fn best_action(&self) -> usize {
        self.action_visits.iter().position_max().unwrap()
    }

    fn update(&mut self, action: usize, mut value: f64) {
        self.action_visits[action] += 1;
        value *= 1. - self.mine_prob[action];

        let visits = self.action_visits[action] as f64;
        let action_value = &mut self.action_values[action];

        *action_value = (*action_value * (visits - 1.) + value) / visits;
    }

    fn value(&self) -> f64 {
        self.action_values
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}

impl<E: EvalFunction, G: Graph> Searcher<E, G> {
    pub fn set_root(&mut self, root: Solver<InternalGame<G>>) {
        self.tree
            .retain(|grid, _| is_grid_subset_of(grid, &root.grid));
        self.root = root;
    }

    pub fn expand(&mut self) {
        let mut rng = rand::thread_rng();
        let mut stack: Vec<(Solver<InternalGame<G>>, usize)> = Vec::new();
        let mut solver = self.root.clone();

        let sols = loop {
            let constraints = solver.solve_csp();

            let Some(node) = self.tree.get(&solver.grid) else {
                break constraints
                    .map(|(groups, subsolutions)| SolutionSet::new(&solver, groups, subsolutions));
            };
            let action = node.next_search_action();

            let mut solver2 = solver.clone();
            solver2.assert_tile(action);
            let Some(sols) = solver2.get_solutionset() else {
                break None;
            };
            let grid = sols.sample_game(&mut rng);

            solver.game.grid = Some(grid);
            solver.uncover_tile(action).unwrap();
            solver.propogate(&mut vec![action]);

            solver2.clear_tile(action);
            stack.push((solver2, action));
        };

        let mut value = match sols {
            Some(sols) => {
                let node = sols.make_treenode(&self.eval);
                let value = node.value();
                self.tree.insert(solver.grid, node);
                value
            }
            None => 1.0,
        };

        for (solver, action) in stack.into_iter().rev() {
            let node = self.tree.get_mut(&solver.grid).unwrap();
            node.update(action, value);
            value = node.value();
        }
    }

    pub fn best_action(&self) -> usize {
        self.tree[&self.root.grid].best_action()
    }

    pub fn training_example(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let node = self.tree.get(&self.root.grid).unwrap();
        let features = self.root.get_solutionset().unwrap().nn_features();
        let mask = grid_as_mask(&self.root.grid);

        let action_values = node
            .action_values
            .iter()
            .zip(node.mine_prob.iter())
            .map(|(&val, &prob)| val / (1. - prob))
            .collect();

        let total_visits = node.action_visits.iter().sum::<usize>() as f64;
        let policy = node
            .action_visits
            .iter()
            .map(|&v| v as f64 / total_visits)
            .collect();

        (features, mask, action_values, policy)
    }
}
