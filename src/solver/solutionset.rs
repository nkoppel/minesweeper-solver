use rand::Rng;
use std::collections::HashMap;
use std::sync::RwLock;
use std::rc::Rc;

use super::csp::*;
use crate::bitvec::*;

use ibig::{UBig, ubig};

pub struct ProbabilityTree {
    total: UBig,
    children: HashMap<usize, ProbabilityTree>
}

pub struct SolutionSet {
    pub(super) subsolutions: Vec<SubSolutionSet>,
    pub(super) num_with_count: Vec<HashMap<usize, usize>>,
    pub(super) solutions_with_count: Vec<HashMap<usize, UBig>>,
    pub(super) count_prob_tree: ProbabilityTree,
    pub(super) mine_probabilities: Vec<UBig>,
    pub(super) unconstrained_probability: UBig,
}

static FACTORIALS: RwLock<Vec<UBig>> = RwLock::new(Vec::new());

pub fn factorial(n: usize) -> UBig {
    let factorials = FACTORIALS.read().expect("RwLock was poisoned");
    if let Some(out) = factorials.get(n) {
        out.clone()
    } else {
        std::mem::drop(factorials);
        let mut factorials = FACTORIALS.write().expect("RwLock was poisoned");

        while factorials.len() <= n {
            let next = factorials.last().map(|x| x * factorials.len()).unwrap_or(ubig!(1));
            factorials.push(next);
        }

        factorials[n].clone()
    }
}

pub fn n_choose_k(n: usize, k: usize) -> UBig {
    if k > n {
        return ubig!(0);
    }
    factorial(n) / (factorial(k) * factorial(n - k))
}

pub struct OuterProductIter<I, T> {
    iters: Vec<I>,
    active_iters: Vec<I>,
    buf: Rc<Vec<T>>
}

impl<I: Iterator<Item = T> + Clone, T> OuterProductIter<I, T> {
    fn new<II: Iterator<Item=I>>(iter: II) -> Self {
        let iters = iter.collect::<Vec<_>>();
        let mut active_iters = iters.clone();
        let buf = active_iters
            .iter_mut()
            .map(|i| i.next())
            // manual implementation of try_collect::<Vec<_>>,
            // but allows us to build on stable
            .try_fold(Vec::new(), |mut out, item| {
                out.push(item?);
                Some(out)
            });

        if let Some(buf) = buf {
            if let Some((active, iter)) = active_iters.last_mut().zip(iters.last()) {
                *active = iter.clone();
            }
            Self {
                iters,
                active_iters,
                buf: Rc::new(buf)
            }
        } else {
            Self {
                iters: Vec::new(),
                active_iters: Vec::new(),
                buf: Rc::new(Vec::new()),
            }
        }
    }
}

impl<I: Iterator<Item = T> + Clone, T: Clone> Iterator for OuterProductIter<I, T> {
    // return an Rc so the compiler will allow us to return a reference
    type Item = Rc<Vec<T>>;

    fn next(&mut self) -> Option<Rc<Vec<T>>> {
        if self.iters.is_empty() {
            return None;
        }

        // copy-on-write: if a previously returned element still exists, this
        // will clone buf internally
        let buf = Rc::make_mut(&mut self.buf);

        let Some((loc, new_elem)) = self.active_iters
            .iter_mut()
            .map(|iter| iter.next())
            .enumerate()
            .rev()
            .find(|(_, elem)| elem.is_some())
            else {
                self.iters.clear();
                self.active_iters.clear();
                buf.clear();
                return None;
            };

        buf[loc] = new_elem.expect("new_elem was not Some");

        #[allow(clippy::needless_range_loop)] // "i" indexes 3 vectors
        for i in loc + 1..self.iters.len() {
            self.active_iters[i] = self.iters[i].clone();
            buf[i] = self.active_iters[i].next().expect("iters contains an empty iterator");
        }

        Some(self.buf.clone())
    }
}

impl ProbabilityTree {
    pub fn with_total(total: UBig) -> Self {
        Self {total, children: HashMap::new()}
    }

    pub fn new() -> Self {
        Self::with_total(ubig!(0))
    }
}

impl SolutionSet {
    /// initializes other fields using "subsolutions" and arguments
    fn init(&mut self, remaining_empty: usize, remaining_mines: usize) {
        let unconstrained_empty = remaining_empty - self.subsolutions[0].mask.len();

        self.num_with_count = self.subsolutions.iter().map(|s| s.get_counts()).collect();
        self.solutions_with_count = vec![HashMap::new(); self.subsolutions.len()];
        self.count_prob_tree = ProbabilityTree::new();

        self.mine_probabilities = vec![ubig!(0); self.subsolutions[0].mask.len()];
        self.unconstrained_probability = ubig!(0);

        println!("{:?}", self.num_with_count);

        for keys in OuterProductIter::new(self.num_with_count.iter().map(|map| map.keys())) {
            let n_mines = keys.iter().copied().sum::<usize>();
            let mut product = self.num_with_count.iter()
                .zip(keys.iter())
                .map(|(map, k)| map[k])
                .fold(ubig!(1), |a, b| a * b);

            if n_mines > remaining_mines {
                product = ubig!(0)
            } else {
                product *= n_choose_k(unconstrained_empty, remaining_mines - n_mines);
            }

            println!("{keys:?} {remaining_mines} {n_mines} {product}");

            let solutions_mut = self.solutions_with_count.iter_mut()
                .zip(keys.iter())
                .map(|(map, k)| map.entry(**k));

            for solutions in solutions_mut {
                *solutions.or_insert_with(|| ubig!(0)) += &product;
            }

            let mut count_prob_tree_ref = &mut self.count_prob_tree;

            for count in keys.iter() {
                count_prob_tree_ref.total += &product;
                count_prob_tree_ref = count_prob_tree_ref.children
                    .entry(**count)
                    .or_insert_with(|| ProbabilityTree::with_total(product.clone()))
            }

            self.unconstrained_probability += product * (remaining_mines - n_mines) / unconstrained_empty;
        }

        let zero = ubig!(0);

        let iter = self.subsolutions
            .iter()
            .zip(self.solutions_with_count.iter()
                .zip(self.num_with_count.iter()));

        println!();
        println!("sol counts {:?}", self.solutions_with_count);

        for (solutions, (solution_counts, num_with_count)) in iter {
            for sol in &solutions.solutions {
                let n_solutions = solution_counts.get(&sol.count_ones()).unwrap_or(&zero)
                    / num_with_count.get(&sol.count_ones()).unwrap_or(&1);

                println!("{sol:?} {n_solutions}");

                for i in sol.iter_ones() {
                    self.mine_probabilities[i] += &n_solutions;
                }
            }
        }
    }

    pub fn new(subsolutions: Vec<SubSolutionSet>, remaining_empty: usize, remaining_mines: usize) -> Self {
        let mut out = Self {
            subsolutions,
            num_with_count: Vec::new(),
            solutions_with_count: Vec::new(),
            count_prob_tree: ProbabilityTree::new(),
            mine_probabilities: Vec::new(),
            unconstrained_probability: ubig!(0),
        };

        if !out.subsolutions.is_empty() {
            out.init(remaining_empty, remaining_mines);
        }
        out
    }

    // returns a soulution uniformly distributed from all remaining solutions
    // on the current board
    pub fn sample<R: Rng + ?Sized>(&mut self, rng: &mut R) -> BitVec {
        let mut count_prob_tree = &mut self.count_prob_tree;
        let mut out = BitVec::new(false, self.subsolutions[0].mask.len());
        let mut i = 0;

        while !count_prob_tree.children.is_empty() {
            let mut rand = rng.gen_range(ubig!(0)..count_prob_tree.total.clone());
            let mut count = None;

            for (k, v) in count_prob_tree.children.iter() {
                if v.total > rand {
                    count = Some(*k);
                    break;
                } else {
                    rand -= &v.total;
                }
            }

            let count = count.unwrap();

            count_prob_tree = count_prob_tree.children.get_mut(&count).unwrap();

            let mut n = rng.gen_range(0..self.num_with_count[i][&count]);

            for sol in &self.subsolutions[i].solutions {
                if sol.count_ones() == count {
                    if n == 0 {
                        out |= sol;
                        break;
                    }
                    n -= 1;
                }
            }

            i += 1;
        }

        out
    }
}
