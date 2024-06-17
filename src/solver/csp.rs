use std::{cmp::Ordering, collections::HashMap};

pub use bitvec::prelude::*;
use smallvec::*;

pub type IntVec = SmallVec<[u8; 64]>;

fn n_choose_k(n: usize, k1: usize) -> usize {
    let k2 = n - k1;
    let (k1, k2) = (k1.min(k2), k1.max(k2));

    let f_k1 = (1..=k1).product::<usize>();
    let f_k2 = f_k1 * (k1 + 1..=k2).product::<usize>();
    let f_n = f_k2 * (k2 + 1..=n).product::<usize>();

    f_n / (f_k1 * f_k2)
}

pub(super) fn solution_count(sol: &[u8], mask: &[u8]) -> usize {
    sol.iter()
        .zip(mask.iter())
        .map(|(&s, &m)| n_choose_k(m as usize, s as usize))
        .product()
}

#[derive(Clone, Debug)]
pub struct SubSolutionSet {
    pub(super) mask: IntVec,
    pub(super) solutions: Vec<IntVec>,
}

fn fill_front(mask: &[u8], vec: &mut IntVec, mut num: usize) {
    for (m, n) in mask.iter().copied().zip(vec.iter_mut()) {
        let add = m - *n;
        *n += (add as usize).min(num) as u8;

        if add as usize > num {
            break;
        }
        num -= add as usize;
    }
}

fn next_combination(mask: &[u8], mut vec: IntVec) -> Option<IntVec> {
    assert_eq!(vec.len(), mask.len());

    let i = vec.iter().position(|v| *v > 0)?;

    let mut zeroed_count = vec[i] as usize;
    vec[i] = 0;
    let mut iter = mask[i + 1..].iter().zip(vec[i + 1..].iter_mut());

    loop {
        let (m, v) = iter.next()?;
        if *v < *m {
            *v += 1;
            break;
        }
        zeroed_count += *v as usize;
        *v = 0;
    }

    fill_front(mask, &mut vec, zeroed_count - 1);

    Some(vec)
}

struct CombinationsIter {
    vec: Option<IntVec>,
    mask: IntVec,
}

impl CombinationsIter {
    fn new(mask: IntVec, num: usize) -> Self {
        let mut vec = smallvec![0; mask.len()];
        fill_front(&mask, &mut vec, num);

        Self {
            mask,
            vec: Some(vec),
        }
    }
}

impl Iterator for CombinationsIter {
    type Item = IntVec;

    fn next(&mut self) -> Option<IntVec> {
        let vec = std::mem::take(&mut self.vec)?;
        self.vec = next_combination(&self.mask, vec.clone());

        Some(vec)
    }
}

pub(super) fn sum_to_usize(vec: &[u8]) -> usize {
    vec.iter().map(|x| *x as usize).sum::<usize>()
}

pub(super) fn intvec_or(v1: &[u8], v2: &[u8]) -> IntVec {
    assert_eq!(v1.len(), v2.len());

    v1.iter().zip(v2.iter()).map(|(&a, &b)| a.max(b)).collect()
}

pub(super) fn intvec_and(v1: &[u8], v2: &[u8]) -> IntVec {
    assert_eq!(v1.len(), v2.len());

    v1.iter().zip(v2.iter()).map(|(&a, &b)| a.min(b)).collect()
}

pub(super) fn equal_on_intersection(intersection: &[u8], a: &[u8], b: &[u8]) -> bool {
    assert_eq!(intersection.len(), a.len());
    assert_eq!(a.len(), b.len());

    intersection
        .iter()
        .zip(Iterator::zip(a.iter(), b.iter()))
        .all(|(i, (a, b))| *i == 0 || *a == *b)
}

fn compare_on_intersection(intersection: &[u8], a: &[u8], b: &[u8]) -> Ordering {
    let a_iter = a.iter().zip(intersection).map(|(s, i)| s.min(i));
    let b_iter = b.iter().zip(intersection).map(|(s, i)| s.min(i));

    a_iter.cmp(b_iter)
}

impl SubSolutionSet {
    pub fn from_constraint(mask: IntVec, count: usize) -> Self {
        let solutions = CombinationsIter::new(mask.clone(), count).collect::<Vec<_>>();

        Self { mask, solutions }
    }

    /// Returns two masks representing which groups are all hints and which are all mines
    pub fn solved_groups(&self) -> (BitVec, BitVec) {
        let mut min_mines: IntVec = smallvec![u8::MAX; self.num_variables()];
        let mut max_mines: IntVec = smallvec![0; self.num_variables()];

        for sol in &self.solutions {
            sol.iter()
                .zip(Iterator::zip(min_mines.iter_mut(), max_mines.iter_mut()))
                .for_each(|(s, (min, max)): (_, (&mut u8, &mut u8))| {
                    *min = (*min).min(*s);
                    *max = (*max).max(*s);
                })
        }

        let all_hints = max_mines
            .iter()
            .zip(self.mask.iter())
            .map(|(&x, &mask)| x == 0 && mask > 0)
            .collect();
        let all_mines = min_mines
            .iter()
            .zip(self.mask.iter())
            .map(|(&x, &mask)| x == mask && mask > 0)
            .collect();

        (all_hints, all_mines)
    }

    /// Returns a mapping from a number of mines to a number of solutions with that number of mines
    pub fn num_solutions_with_num_mines(&self) -> HashMap<usize, usize> {
        let mut out = HashMap::new();

        for (count, s) in self.solutions.iter().map(|s| (sum_to_usize(s), s)) {
            *out.entry(count).or_insert(0) += solution_count(s, &self.mask);
        }

        out
    }

    pub fn num_solutions(&self) -> usize {
        self.solutions.len()
    }

    pub fn num_variables(&self) -> usize {
        self.mask.len()
    }

    /// Remove all variables with vars[i] > 0 from self
    fn remove_variables(&mut self, vars: &[u8]) {
        for (m, v) in self.mask.iter_mut().zip(vars.iter()) {
            if *v > 0 {
                *m = 0;
            }
        }

        for sol in self.solutions.iter_mut() {
            for (s, v) in sol.iter_mut().zip(vars.iter()) {
                if *v > 0 {
                    *s = 0;
                }
            }
        }
    }

    fn proven_on_intersection(&self, intersection: &[u8]) -> bool {
        let Some(first) = self.solutions.first() else {
            return true;
        };

        self.solutions[1..]
            .iter()
            .all(|sol| equal_on_intersection(intersection, first, sol))
    }

    fn group_on_mask<'a>(
        &'a mut self,
        mask: &'a [u8],
    ) -> impl Iterator<Item = &[SmallVec<[u8; 64]>]> + 'a {
        self.solutions
            .sort_unstable_by(|a, b| compare_on_intersection(mask, a, b));
        self.solutions
            .chunk_by(|a, b| equal_on_intersection(mask, a, b))
    }

    pub fn try_merge(&mut self, mut other: Self) -> Result<(), Self> {
        let mask = intvec_or(&self.mask, &other.mask);
        let intersection = intvec_and(&self.mask, &other.mask);

        let self_proven = self.proven_on_intersection(&intersection);
        let other_proven = other.proven_on_intersection(&intersection);

        if self_proven || other_proven {
            if self_proven {
                std::mem::swap(self, &mut other);
            }

            if let Some(first) = other.solutions.first() {
                self.solutions
                    .retain(|sol| equal_on_intersection(&intersection, first, sol));
            }
            self.remove_variables(&intersection);

            if sum_to_usize(&self.mask) == 0 {
                *self = other;
                return Ok(());
            } else {
                return Err(other);
            }
        }

        let mut solutions = Vec::new();

        // Alternative way to merge groups that should be faster in theory but ends up being
        // slower in practice. Keeping this around for the sake of it.

        // let mut groups1 = self.group_on_mask(&intersection).peekable();
        // let mut groups2 = other.group_on_mask(&intersection).peekable();

        // loop {
        // let (Some(&group1), Some(&group2)) = (groups1.peek(), groups2.peek()) else {
        // break;
        // };

        // match compare_on_intersection(&intersection, &group1[0], &group2[0]) {
        // Ordering::Less => {
        // groups1.next();
        // continue;
        // },
        // Ordering::Greater => {
        // groups2.next();
        // continue;
        // },
        // Ordering::Equal => {},
        // }

        // for sol1 in group1 {
        // for sol2 in group2 {
        // solutions.push(intvec_or(sol1, sol2));
        // }
        // }

        // groups1.next();
        // groups2.next();
        // }

        // std::mem::drop(groups1);
        // std::mem::drop(groups2);

        for sol1 in &self.solutions {
            for sol2 in &other.solutions {
                if equal_on_intersection(&intersection, sol1, sol2) {
                    solutions.push(intvec_or(sol1, sol2))
                }
            }
        }

        *self = Self { mask, solutions };
        Ok(())
    }
}

pub fn merge_all_subsolutions(sols: &mut Vec<SubSolutionSet>) {
    let mut i = 0;

    while i < sols.len() {
        let pos = sols
            .iter()
            .enumerate()
            .skip(i + 1)
            .map(|(j, sol)| {
                (
                    sol.mask
                        .iter()
                        .zip(sols[i].mask.iter())
                        .map(|(&a, &b)| a.min(b) as usize)
                        .sum::<usize>(),
                    j,
                )
            })
            .max();

        if let Some((1.., j)) = pos {
            let tmp = sols.swap_remove(j);

            if let Err(tmp) = sols[i].try_merge(tmp) {
                sols.push(tmp);
            }
        } else {
            i += 1;
        }
    }
}

pub fn solved_groups(subsolutions: &[SubSolutionSet]) -> (BitVec, BitVec) {
    let Some(num_vars) = subsolutions.first().map(|s| s.num_variables()) else {
        return (BitVec::new(), BitVec::new());
    };

    let mut all_hints = bitvec![usize, Lsb0; 0; num_vars];
    let mut all_mines = bitvec![usize, Lsb0; 0; num_vars];

    for sol in subsolutions {
        let (hints, mines) = sol.solved_groups();

        all_hints |= hints;
        all_mines |= mines;
    }
    (all_hints, all_mines)
}
