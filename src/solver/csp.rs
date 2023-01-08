use std::collections::HashMap;
use crate::bitvec::BitVec;

#[derive(Clone, Debug)]
pub struct SubSolutionSet {
    pub(super) mask: BitVec,
    pub(super) solutions: Vec<BitVec>,
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

    intersection
        .iter_elems()
        .zip(Iterator::zip(a.iter_elems(), b.iter_elems()))
        .all(|(i, (a, b))| a & i == b & i)
}

impl SubSolutionSet {
    pub fn from_constraint(mask: BitVec, count: usize) -> Self {
        let n_constrained = mask.count_ones();

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
        if self.len() == 0 {
            return (BitVec::new(false, self.variables()), BitVec::new(false, self.variables()));
        }
        let mut mines = self.mask.clone();
        let mut safe = self.mask.clone();

        safe.invert_inplace();

        for solution in &self.solutions {
            mines &= solution;
            safe |= solution;
        }

        safe.invert_inplace();

        (mines, safe)
    }

    pub fn get_counts(&self) -> HashMap<usize, usize> {
        let mut out = HashMap::new();

        for count in self.solutions.iter().map(|s| s.count_ones()) {
            *out.entry(count).or_insert(0) += 1;
        }

        out
    }

    pub fn len(&self) -> usize {
        self.solutions.len()
    }

    pub fn variables(&self) -> usize {
        self.mask.len()
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

        // if solutions.len() == 0 {
            // println!("{self:?}");
            // println!("{other:?}");
            // println!("{solutions:?}");
            // panic!();
        // }

        // println!("merge {} {} {}", self.mask.count_ones(), other.mask.count_ones(), mask.count_ones());

        Self { mask, solutions }
    }
}

pub fn merge_all_subsolutions(sols: &mut Vec<SubSolutionSet>) -> Option<(BitVec, BitVec)> {
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
            sols[i] = sols[i].merge(&sols[j]);
            sols.swap_remove(j);
        } else {
            i += 1;
        }
    }

    let mut mines = BitVec::new(false, sols.get(0)?.variables());
    let mut safe = BitVec::new(false, sols.get(0)?.variables());

    // println!();

    for sol in sols.iter() {
        let (m, s) = sol.get_solved();

        // println!("{} {m:?} {s:?}", sol.len());

        mines |= &m;
        safe |= &s;
    }

    if safe.count_ones() > 0 {
        Some((mines, safe))
    } else {
        None
    }
}
