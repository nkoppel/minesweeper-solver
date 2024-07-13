use smallvec::SmallVec;

use super::*;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrecomputedGraph {
    neighbors: Rc<[SmallVec<[usize; 8]>]>,
}

impl PrecomputedGraph {
    pub fn from_graph(graph: &impl Graph) -> Self {
        Self {
            neighbors: (0..graph.num_tiles())
                .map(|i| graph.neighbors(i).collect())
                .collect(),
        }
    }

    pub fn from_adjacency_list(list: &[impl AsRef<[usize]>]) -> Self {
        Self {
            neighbors: list
                .iter()
                .map(|slice| SmallVec::from_slice(slice.as_ref()))
                .collect(),
        }
    }
}

impl Graph for PrecomputedGraph {
    fn neighbors(&self, pos: usize) -> impl Iterator<Item = usize> + '_ {
        self.neighbors[pos].iter().copied()
    }

    fn num_tiles(&self) -> usize {
        self.neighbors.len()
    }
}
