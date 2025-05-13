use super::*;

use malachite::{
    base::num::{arithmetic::traits::Factorial, conversion::traits::RoundingInto},
    base::rounding_modes::RoundingMode,
    rational::Rational,
    Natural,
};

fn n_choose_k<T>(n: u64, k: u64) -> T
where
    T: Factorial + std::ops::Mul<Output = T> + std::ops::Div<Output = T>,
{
    T::factorial(n) / (T::factorial(k) * T::factorial(n - k))
}

impl ArrangementSet {
    fn count_subsolutions(&self, groups: &[BitSet], group_sizes: &[usize]) -> Vec<(usize, u64)> {
        let mut counts = vec![0; self.mask.count_ones() + 1];

        for arrangement in &self.arrangements {
            let count: u64 = self
                .groups
                .iter_ones()
                .map(|group| {
                    let group_mines = groups[group].count_overlap_ones(arrangement);
                    let group_size = group_sizes[group];
                    n_choose_k::<u64>(group_size as u64, group_mines as u64)
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
        num_tiles: usize,
    ) -> Vec<(usize, Vec<u64>)> {
        let mut counts = vec![vec![0; num_tiles]; num_tiles + 1];

        for arrangement in &self.arrangements {
            let count: u64 = self
                .groups
                .iter_ones()
                .map(|group| {
                    let group_mines = groups[group].count_overlap_ones(arrangement);
                    let group_size = group_sizes[group];
                    n_choose_k::<u64>(group_size as u64, group_mines as u64)
                })
                .product();

            let total_mines = arrangement.count_ones();

            for group in self.groups.iter_ones() {
                let group_mines = groups[group].count_overlap_ones(arrangement);
                let group_size = group_sizes[group];

                for tile in groups[group].iter_ones() {
                    counts[total_mines][tile] +=
                        count * (group_size - group_mines) as u64 / group_size as u64;
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

pub fn natural_ratio_as_float(n: &Natural, d: &Natural) -> f64 {
    Rational::from_naturals_ref(n, d)
        .rounding_into(RoundingMode::Nearest)
        .0
}

impl MineArrangements {
    fn solution_counts(&self) -> Vec<(Natural, Vec<(usize, u64)>)> {
        let group_sizes = self.groups.iter().map(BitSet::count_ones).collect_vec();
        let num_unconstrained = self.uncontrained_empties().count_ones();

        self.sub_arrangements
            .iter()
            .map(|sa| sa.count_subsolutions(&self.groups, &group_sizes))
            .collect_vec()
            .iter()
            .map(|i| i.iter().copied())
            .multi_cartesian_product()
            .filter_map(move |counts| {
                let unconstrained_mines = self
                    .remaining_mines
                    .checked_sub(counts.iter().map(|x| x.0).sum::<usize>())?;

                    if unconstrained_mines > num_unconstrained {
                        return None;
                    }

                let constrained_solutions: Natural =
                    counts.iter().map(|x| Natural::from(x.1)).product();
                let unconstrained_solutions: Natural =
                    n_choose_k(num_unconstrained as u64, unconstrained_mines as u64);

                Some((constrained_solutions * unconstrained_solutions, counts))
            })
            .collect()
    }

    pub fn total_solutions(&self) -> Natural {
        self.solution_counts().into_iter().map(|(num, _)| num).sum()
    }

    pub fn tile_safe_solutions(&self) -> Vec<Natural> {
        let group_sizes = self.groups.iter().map(BitSet::count_ones).collect_vec();
        let num_unconstrained = self.uncontrained_empties().count_ones();

        let subsolution_info: Vec<Vec<(usize, u64, Vec<u64>)>> = self
            .sub_arrangements
            .iter()
            .map(|sa| {
                let counts = sa.count_subsolutions(&self.groups, &group_sizes);
                let tile_mine_counts =
                    sa.count_tile_safes(&self.groups, &group_sizes, self.num_tiles);

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
            let Some(unconstrained_mines) = self
                .remaining_mines
                .checked_sub(counts.iter().map(|x| x.0).sum::<usize>())
            else {
                continue;
            };

            if unconstrained_mines > num_unconstrained {
                continue;
            }

            let unconstrained_solutions: Natural =
                n_choose_k(num_unconstrained as u64, unconstrained_mines as u64);
            let constrained_solutions: Natural =
                counts.iter().map(|x| Natural::from(x.1)).product();

            let total_solutions = &unconstrained_solutions * &constrained_solutions;

            for (_, solutions, counts) in counts {
                for (c1, c2) in out.iter_mut().zip(counts) {
                    if *c2 > 0 {
                        *c1 += &total_solutions * Natural::from(*c2) / Natural::from(*solutions);
                    }
                }
            }

            if num_unconstrained > 0 {
                unconstrained_count += total_solutions
                    * Natural::from(num_unconstrained - unconstrained_mines)
                    / Natural::from(num_unconstrained);
            }
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
            .map(|sol| natural_ratio_as_float(sol, &total_solutions))
            .collect()
    }
}
