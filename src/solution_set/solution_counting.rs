use super::*;

use malachite::{
    num::{arithmetic::traits::Factorial, conversion::traits::RoundingInto},
    rounding_modes::RoundingMode,
    Natural, Rational,
};

fn n_choose_k(n: usize, k1: usize) -> usize {
    let k2 = n - k1;
    let (k1, k2) = (k1.min(k2), k1.max(k2));

    let f_k1 = (1..=k1).product::<usize>();
    let f_k2 = f_k1 * (k1 + 1..=k2).product::<usize>();
    let f_n = f_k2 * (k2 + 1..=n).product::<usize>();

    f_n / (f_k1 * f_k2)
}

fn n_choose_k_natural(n: u64, k: u64) -> Natural {
    Natural::factorial(n) / (Natural::factorial(k) * Natural::factorial(n - k))
}

impl ArrangementSet {
    fn count_subsolutions(&self, groups: &[BitSet], group_sizes: &[usize]) -> Vec<(usize, usize)> {
        let mut counts = vec![0; self.mask.count_ones() + 1];

        for arrangement in &self.arrangements {
            let count: usize = self
                .groups
                .iter_ones()
                .map(|group| {
                    let group_mines = groups[group].count_overlap_ones(arrangement);
                    let group_size = group_sizes[group];
                    n_choose_k(group_size, group_mines)
                })
                .product();

            counts[arrangement.count_ones()] += count;
        }

        counts
            .into_iter()
            .enumerate()
            .filter(|(_, x)| *x > 0)
            .collect()
    }

    fn count_tile_safes(
        &self,
        groups: &[BitSet],
        group_sizes: &[usize],
    ) -> Vec<(usize, Vec<usize>)> {
        let mut counts = vec![vec![0; self.mask.bits()]; self.mask.bits() + 1];

        for arrangement in &self.arrangements {
            let count: usize = self
                .groups
                .iter_ones()
                .map(|group| {
                    let group_mines = groups[group].count_overlap_ones(arrangement);
                    let group_size = group_sizes[group];
                    n_choose_k(group_size, group_mines)
                })
                .product();

            let total_mines = arrangement.count_ones();

            for group in self.groups.iter_ones() {
                let group_mines = groups[group].count_overlap_ones(arrangement);
                let group_size = group_sizes[group];

                for tile in groups[group].iter_ones() {
                    counts[total_mines][tile] += count * (group_size - group_mines) / group_size;
                }
            }
        }

        counts
            .into_iter()
            .enumerate()
            .filter(|(_, grid)| grid.iter().any(|c| *c > 0))
            .collect()
    }
}

impl MineArrangements {
    pub fn total_solutions(&self) -> Natural {
        let group_sizes = self.groups.iter().map(BitSet::count_ones).collect_vec();
        let num_unconstrained = self.uncontrained_empties().count_ones();

        self.sub_arrangements
            .iter()
            .map(|sa| sa.count_subsolutions(&self.groups, &group_sizes))
            .multi_cartesian_product()
            .map(|counts| {
                let unconstrained_mines =
                    self.remaining_mines - counts.iter().map(|x| x.0).sum::<usize>();

                let constrained_solutions: Natural =
                    counts.iter().map(|x| Natural::from(x.1)).product();
                let unconstrained_solutions =
                    n_choose_k_natural(num_unconstrained as u64, unconstrained_mines as u64);

                constrained_solutions * unconstrained_solutions
            })
            .sum()
    }

    pub fn tile_safe_solutions(&self) -> Vec<Natural> {
        let group_sizes = self.groups.iter().map(BitSet::count_ones).collect_vec();
        let num_unconstrained = self.uncontrained_empties().count_ones();

        let subsolution_info: Vec<Vec<(usize, usize, Vec<usize>)>> = self
            .sub_arrangements
            .iter()
            .map(|sa| {
                let counts = sa.count_subsolutions(&self.groups, &group_sizes);
                let tile_mine_counts = sa.count_tile_safes(&self.groups, &group_sizes);

                counts
                    .into_iter()
                    .zip(tile_mine_counts)
                    .map(|((i, count), (_, tile_mines))| (i, count, tile_mines))
                    .collect()
            })
            .collect();

        let mut out = vec![Natural::from(0u32); self.num_tiles];
        let mut unconstrained_count = Natural::from(0u32);

        for counts in subsolution_info.iter().multi_cartesian_product() {
            let unconstrained_mines =
                self.remaining_mines - counts.iter().map(|x| x.0).sum::<usize>();

            let unconstrained_solutions =
                n_choose_k_natural(num_unconstrained as u64, unconstrained_mines as u64);
            let constrained_solutions: Natural =
                counts.iter().map(|x| Natural::from(x.1)).product();

            let total_solutions = &unconstrained_solutions * &constrained_solutions;

            for (_, solutions, counts) in counts {
                for (c1, c2) in out.iter_mut().zip(counts) {
                    *c1 += &total_solutions * Natural::from(*c2) / Natural::from(*solutions);
                }
            }

            unconstrained_count += total_solutions
                * Natural::from(num_unconstrained - unconstrained_mines)
                / Natural::from(num_unconstrained);
        }

        for tile in self.uncontrained_empties().iter_ones() {
            out[tile] = unconstrained_count.clone();
        }

        out
    }

    pub fn tile_safe_probability(&self) -> Vec<f64> {
        let solutions = self.tile_safe_solutions();
        let total_solutions = self.total_solutions();

        solutions
            .iter()
            .map(|sol| {
                Rational::from_naturals_ref(sol, &total_solutions)
                    .rounding_into(RoundingMode::Nearest)
                    .0
            })
            .collect()
    }
}
