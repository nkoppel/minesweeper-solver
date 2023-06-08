use std::collections::HashMap;

pub use bitvec::prelude::*;
use smallvec::*;

type MaskVec = SmallVec<[bool; 64]>;
type IntVec = SmallVec<[u8; 64]>;

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

fn sum_to_usize(vec: &[u8]) -> usize {
    vec.iter().map(|x| *x as usize).sum::<usize>()
}

fn intvec_or(v1: &[u8], v2: &[u8]) -> IntVec {
    assert_eq!(v1.len(), v2.len());

    v1.iter().zip(v2.iter()).map(|(&a, &b)| a.max(b)).collect()
}

fn intvec_and(v1: &[u8], v2: &[u8]) -> IntVec {
    assert_eq!(v1.len(), v2.len());

    v1.iter().zip(v2.iter()).map(|(&a, &b)| a.min(b)).collect()
}

pub fn equal_on_intersection(intersection: &[u8], a: &[u8], b: &[u8]) -> bool {
    assert_eq!(intersection.len(), a.len());
    assert_eq!(a.len(), b.len());

    intersection
        .iter()
        .zip(Iterator::zip(a.iter(), b.iter()))
        .all(|(i, (a, b))| *i == 0 || *a == *b)
}

impl SubSolutionSet {
    pub fn from_constraint(mask: IntVec, count: usize) -> Self {
        let solutions = CombinationsIter::new(mask.clone(), count).collect::<Vec<_>>();

        Self { mask, solutions }
    }

    /// Returns two masks representing which groups are all hints and which are all mines
    pub fn solved_groups(&self) -> (BitVec, BitVec) {
        let mut min_mines: IntVec = smallvec![u8::MAX; self.variables()];
        let mut max_mines: IntVec = smallvec![0; self.variables()];

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

    /// Returns a mapping from a number of mines contained within this solution to the number of
    /// solutions with that number of mines
    pub fn get_counts(&self) -> HashMap<usize, usize> {
        let mut out = HashMap::new();

        for count in self.solutions.iter().map(|v| sum_to_usize(v)) {
            *out.entry(count).or_insert(0) += 1;
        }

        out
    }

    pub fn num_solutions(&self) -> usize {
        self.solutions.len()
    }

    pub fn variables(&self) -> usize {
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
        let Some(first) = self.solutions.get(0) else { return true };

        self.solutions[1..]
            .iter()
            .all(|sol| equal_on_intersection(intersection, first, sol))
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

            if let Some(first) = other.solutions.get(0) {
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

        // TODO: It may be possible to make this faster with a HashMap
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
    let num_vars = subsolutions[0].variables();

    let mut all_hints = bitvec![usize, Lsb0; 0; num_vars];
    let mut all_mines = bitvec![usize, Lsb0; 0; num_vars];

    for sol in subsolutions {
        let (hints, mines) = sol.solved_groups();

        all_hints |= hints;
        all_mines |= mines;
    }
    (all_hints, all_mines)
}
