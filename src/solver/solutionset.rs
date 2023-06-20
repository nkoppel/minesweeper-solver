use std::collections::HashMap;
use std::sync::RwLock;

use super::{csp::*, *};
use crate::game::*;

use ibig::{ubig, UBig};
use itertools::Itertools;
use rand::prelude::*;
use smallvec::*;

pub struct SolutionSet {
    pub grid: Vec<Tile>,
    pub(super) groups: Vec<Vec<usize>>,
    pub(super) subsolutions: Vec<SubSolutionSet>,
    pub(super) num_solutions_with_num_mines: Vec<HashMap<usize, usize>>,
    pub(super) subsolution_mine_counts: Vec<(Vec<usize>, UBig, UBig)>,
    pub(super) total_num_solutions: UBig,
    pub(super) remaining_mines: usize,
    pub(super) remaining_empties: usize,
}

static FACTORIALS: RwLock<Vec<UBig>> = RwLock::new(Vec::new());

fn factorial(n: usize) -> UBig {
    let factorials = FACTORIALS.read().expect("RwLock was poisoned");

    if let Some(out) = factorials.get(n) {
        out.clone()
    } else {
        std::mem::drop(factorials);
        let mut factorials = FACTORIALS.write().expect("RwLock was poisoned");

        while factorials.len() <= n {
            let next = factorials
                .last()
                .map(|x| x * factorials.len())
                .unwrap_or(ubig!(1));
            factorials.push(next);
        }

        factorials[n].clone()
    }
}

fn n_choose_k(n: usize, k: usize) -> UBig {
    if k > n {
        return ubig!(0);
    }
    factorial(n) / (factorial(k) * factorial(n - k))
}

fn ubig_ratio_to_float(mut n: UBig, mut d: UBig) -> f64 {
    let shift = n.bit_len().min(d.bit_len()).saturating_sub(64);
    n >>= shift;
    d >>= shift;
    n.to_f64() / d.to_f64()
}

impl SolutionSet {
    pub fn new(
        solver: &Solver<impl Game>,
        groups: Vec<Vec<usize>>,
        subsolutions: Vec<SubSolutionSet>,
    ) -> Self {
        let remaining_empties = solver.remaining_empty_tiles();
        let remaining_mines = solver.remaining_mines();

        let num_solutions_with_num_mines = subsolutions
            .iter()
            .map(SubSolutionSet::num_solutions_with_num_mines)
            .collect::<Vec<_>>();

        let constrained_empties = subsolutions
            .iter()
            .map(|sol| sum_to_usize(&sol.mask))
            .sum::<usize>();
        let unconstrained_empties = remaining_empties - constrained_empties;
        let mut prefix_sum = ubig!(0);

        let subsolution_mine_counts = num_solutions_with_num_mines
            .iter()
            .map(|map| map.iter().map(|(&k, &v)| (k, v)))
            .multi_cartesian_product()
            .filter_map(|counts| {
                let constrainted_mines = counts.iter().map(|c| c.0).sum::<usize>();
                let unconstrainted_mines = remaining_mines.checked_sub(constrainted_mines)?;

                let n_solutions = n_choose_k(unconstrained_empties, unconstrainted_mines)
                    * counts
                        .iter()
                        .map(|c| UBig::from_le_bytes(&c.1.to_le_bytes()))
                        .fold(ubig!(1), |product, x| product * x);

                let old_prefix_sum = prefix_sum.clone();
                let counts = counts.iter().map(|c| c.0).collect();
                prefix_sum += &n_solutions;
                Some((counts, n_solutions, old_prefix_sum))
            })
            .collect();

        SolutionSet {
            grid: solver.grid.clone(),
            groups,
            subsolutions,
            num_solutions_with_num_mines,
            subsolution_mine_counts,
            total_num_solutions: prefix_sum,
            remaining_mines,
            remaining_empties,
        }
    }

    fn unconstrained_mine_probability(&self) -> f64 {
        let constrained_empties = self
            .subsolutions
            .iter()
            .map(|sol| sum_to_usize(&sol.mask))
            .sum::<usize>();
        let unconstrained_empties = self.remaining_empties - constrained_empties;

        self.subsolution_mine_counts
            .iter()
            .map(|(counts, num_solutions, _)| {
                let constrained_mines = counts.iter().sum::<usize>();
                let unconstrained_mines = self.remaining_mines - constrained_mines;

                let weight =
                    ubig_ratio_to_float(num_solutions.clone(), self.total_num_solutions.clone());
                weight * unconstrained_mines as f64 / unconstrained_empties as f64
            })
            .sum()
    }

    fn group_mine_probabilities(&self) -> Vec<f64> {
        // mapping from subsolution mine count to number of solutions
        let mut total_solutions_with_num_mines: Vec<HashMap<usize, f64>> =
            vec![HashMap::new(); self.subsolutions.len()];

        for (counts, num_solutions, _) in self.subsolution_mine_counts.iter() {
            for (id, count) in counts.iter().enumerate() {
                let ratio =
                    ubig_ratio_to_float(num_solutions.clone(), self.total_num_solutions.clone());

                *total_solutions_with_num_mines[id]
                    .entry(*count)
                    .or_default() += ratio;
            }
        }

        // for each subsolution,
        // (count / mask) * (proportion of solutions represented by the current configuration)
        self.subsolutions
            .iter()
            .zip(total_solutions_with_num_mines.iter())
            .zip(self.num_solutions_with_num_mines.iter())
            .flat_map(|((sol, probs), solutions_with_count)| {
                sol.solutions.iter().filter_map(move |s| {
                    let count = sum_to_usize(s);

                    let num_solutions = solution_count(s, &sol.mask);
                    let num_total_solutions = solutions_with_count[&count];

                    let prob =
                        num_solutions as f64 / num_total_solutions as f64 * probs.get(&count)?;

                    Some(
                        s.iter()
                            .zip(sol.mask.iter())
                            .map(move |(&s, &m)| s as f64 / m as f64 * prob),
                    )
                })
            })
            .fold(
                vec![0.; self.subsolutions[0].mask.len()],
                |mut sum, iter| {
                    for (s, x) in sum.iter_mut().zip(iter) {
                        if x.is_finite() {
                            *s += x;
                        }
                    }
                    sum
                },
            )
    }

    pub fn tile_mine_probabilities(&self) -> Vec<f64> {
        let unconstrained_prob = self.unconstrained_mine_probability();
        let group_probs = self.group_mine_probabilities();

        let mut out = self
            .grid
            .iter()
            .map(|tile| match tile {
                Empty => unconstrained_prob,
                Mine { .. } => 1.0,
                AssertHint { .. } => 0.0,
                Hint { .. } => 0.0,
            })
            .collect::<Vec<_>>();

        for (group, prob) in self.groups.iter().zip(group_probs.iter()) {
            for g in group {
                out[*g] = *prob;
            }
        }

        out
    }

    fn sample(&self, rng: &mut impl Rng) -> IntVec {
        let solution_num = rng.gen_range(ubig!(0)..self.total_num_solutions.clone());
        let counts_idx = self
            .subsolution_mine_counts
            .binary_search_by_key(&&solution_num, |tup| &tup.2)
            .unwrap_or_else(|i| i - 1);

        let counts = &self.subsolution_mine_counts[counts_idx].0;

        self.subsolutions
            .iter()
            .zip(counts.iter())
            .zip(self.num_solutions_with_num_mines.iter())
            .map(|((sol, &count), num_with_count)| {
                let num_solutions = num_with_count[&count];
                let mut sample = rng.gen_range(0..num_solutions) as isize;

                for s in sol.solutions.iter() {
                    let s_count = sum_to_usize(s);

                    if s_count != count {
                        continue;
                    }

                    sample -= solution_count(s, &sol.mask) as isize;

                    if sample < 0 {
                        return s;
                    }
                }

                unreachable!()
            })
            .fold(
                smallvec![0; self.subsolutions[0].num_variables()],
                |state, x| intvec_or(&state, x),
            )
    }

    pub fn sample_game(&self, rng: &mut impl Rng) -> BitVec {
        let mut out: BitVec = self.grid.iter().map(|c| matches!(c, Mine { .. })).collect();
        let sample = self.sample(rng);

        for (num_mines, group) in sample.iter().zip(self.groups.iter()) {
            for i in n_unique_random(group.len(), *num_mines as usize, rng) {
                out.set(group[i], true);
            }
        }

        let mut unconstrained_tiles = self.grid.iter().map(|c| *c == Empty).collect::<BitVec>();

        for tile in self.groups.iter().flatten() {
            unconstrained_tiles.set(*tile, false);
        }

        let unconstrained_tiles = unconstrained_tiles.iter_ones().collect::<Vec<_>>();
        let unconstrained_mines = self.remaining_mines - sum_to_usize(&sample);

        for i in n_unique_random(unconstrained_tiles.len(), unconstrained_mines, rng) {
            out.set(unconstrained_tiles[i], true);
        }

        out
    }
}
