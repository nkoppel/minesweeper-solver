use std::collections::HashMap;

use crate::solver::*;

struct TreeNode {
    visits: usize,
    value: f64,
    mine_prob: Vec<f64>,
    policy: Vec<f64>,
    action_visits: Vec<usize>,
}

struct Searcher {
    tree: HashMap<Vec<Square>, TreeNode>,
}
