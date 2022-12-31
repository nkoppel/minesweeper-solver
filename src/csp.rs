use std::collections::HashMap;
use crate::bitvec::BitVec;

#[derive(Clone, Debug)]
pub struct SubSolutionSet {
    mask: BitVec,
    solutions: Vec<BitVec>,
}

pub struct SolutionSet {
    subsolutions: Vec<SubSolutionSet>,
    // TODO: use bigints
    subsolution_count_probabilities: Vec<HashMap<usize, f64>>,
    mine_probabilities: Vec<f64>,
}

pub struct CombinationsIter {
    combination: u128,
    max: u128,
}

impl CombinationsIter {
    pub fn new(max: u128, num_set: usize) -> Self {
        Self {
            combination: (1 << num_set) - 1,
            max,
        }
    }
}

impl Iterator for CombinationsIter {
    type Item = u128;

    fn next(&mut self) -> Option<u128> {
        if self.combination > self.max {
            return None;
        }

        let v = self.combination;

        // Adapted from: https://graphics.stanford.edu/~seander/bithacks.html#NextBitPermutation
        let t = v | (v - 1); // t gets v's least significant 0 bits set to 1
        // Next set to 1 the most significant bit to change, 
        // set to 0 the least significant ones, and add the necessary 1 bits.
        self.combination = (t + 1) | (((!t & (t + 1)) - 1) >> (v.trailing_zeros() + 1));

        Some(v)
    }
}

fn equal_on_intersection(intersection: &BitVec, a: &BitVec, b: &BitVec) -> bool {
    // println!("{intersection:?}");
    // println!("{a:?}");
    // println!("{b:?}");
    for (i, (a, b)) in intersection.iter_elems().zip(Iterator::zip(a.iter_elems(), b.iter_elems())) {
        if a & i != b & i {
            // println!("f\n");
            return false;
        }
    }

    // println!("t\n");
    return true;
}

impl SubSolutionSet {
    pub fn from_constraint(mask: BitVec, count: usize) -> Self {
        let n_constrained = mask.count_ones();

        // println!("{n_constrained}");

        let solutions = CombinationsIter::new((1 << n_constrained) - 1, count)
            .map(|bits| mask.deposit_bits(bits))
            .collect::<Vec<_>>();

        Self {
            mask,
            solutions
        }
    }

    /// Outputs two masks representing whether each variable
    /// is proveably 1 and proveably 0
    pub fn get_solved(&self) -> (BitVec, BitVec) {
        let mut solved_ones = self.mask.clone();
        let mut solved_zeros = self.mask.clone();

        solved_zeros.invert_inplace();

        for solution in &self.solutions {
            solved_ones &= solution;
            solved_zeros |= solution;
        }

        solved_zeros.invert_inplace();

        (solved_ones, solved_zeros)
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mask = &self.mask | &other.mask;
        let intersection = &self.mask & &other.mask;
        let mut solutions = Vec::new();

        for sol1 in &self.solutions {
            for sol2 in &other.solutions {
                if equal_on_intersection(&intersection, sol1, sol2) {
                    solutions.push(sol1 | sol2)
                }
            }
        }

        Self { mask, solutions }
    }
}

pub fn merge_all_subsolutions(sols: &mut Vec<SubSolutionSet>) {
    let mut i = 0;

    while i < sols.len() {
        let pos = sols
            .iter()
            .enumerate()
            .skip(i + 1)
            .map(|(j, sol)| ((&sol.mask & &sols[i].mask).count_ones(), j))
            .filter(|(count, _)| *count > 0)
            .max();

        if let Some((_, j)) = pos {
            // print!("{} {} ", sols[i].solutions.len(), sols[j].solutions.len());
            sols[i] = sols[i].merge(&sols[j]);
            // println!("{}", sols[i].solutions.len());
            sols.swap_remove(j);
        } else {
            i += 1;
        }

        // println!();
        // println!("{sols:?}");
    }
}
