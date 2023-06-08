use std::collections::HashMap;
use ibig::UBig;

use crate::solver::*;

struct Action {
    safe_prob: f64,
    location: (usize, usize),
}

struct TreeNode {
    value: f64,
    visits: usize,
    actions: Vec<Action>,
}

struct Searcher {
    tree: HashMap<Vec<Vec<Square>>, TreeNode>
}
