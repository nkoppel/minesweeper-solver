use itertools::{iproduct, Itertools};
use malachite::{num::arithmetic::traits::Factorial, Natural};

use crate::{bitset::BitSet, board::*, game::*};

#[derive(Clone, Debug)]
struct ArrangementSet {
    mask: BitSet,
    groups: BitSet,
    arrangements: Vec<BitSet>,
}

#[derive(Clone, Debug)]
pub struct MineArrangements {
    groups: Vec<BitSet>,
    sub_arrangements: Vec<ArrangementSet>,
    mask: BitSet,
    remaining_mines: usize,
    num_tiles: usize,
}

struct CombinationsIter<'a> {
    group_fills: &'a [Vec<BitSet>],
    groups: &'a [usize],
    next: Option<BitSet>,
}

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

fn fill_front(
    group_fills: &[impl AsRef<[BitSet]>],
    groups: &[usize],
    combination: &mut BitSet,
    mut num_ones: usize,
) {
    for group in groups.iter().map(|g| group_fills[*g].as_ref()) {
        let group_size = group.len() - 1;

        if group_size >= num_ones {
            *combination += &group[num_ones];
            break;
        }

        *combination += group.last().unwrap();
        num_ones -= group_size;
    }
}

fn next_combination(
    group_fills: &[Vec<BitSet>],
    groups: &[usize],
    mut combination: BitSet,
) -> Option<BitSet> {
    let start_pos = groups
        .iter()
        .position(|group| combination.overlaps_with(group_fills[*group].last().unwrap()))?;
    let start_group = group_fills[groups[start_pos]].last().unwrap();

    let mut zeroed_count = start_group.count_overlap_ones(&combination);
    combination -= start_group;

    for group_idx in &groups[start_pos + 1..groups.len()] {
        let group = group_fills[*group_idx].last().unwrap();

        if !group.is_subset_of(&combination) {
            let ones = combination.count_overlap_ones(group);
            combination += &group_fills[*group_idx][ones + 1];
            fill_front(group_fills, groups, &mut combination, zeroed_count - 1);
            return Some(combination);
        }

        zeroed_count += combination.count_overlap_ones(group);
        combination -= group;
    }

    None
}

impl<'a> CombinationsIter<'a> {
    fn new(group_fills: &'a [Vec<BitSet>], groups: &'a [usize], num: usize) -> Self {
        let mut next = BitSet::empty(group_fills[0][0].bits());
        fill_front(group_fills, groups, &mut next, num);

        Self {
            group_fills,
            groups,
            next: Some(next),
        }
    }
}

impl Iterator for CombinationsIter<'_> {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        let next = next_combination(self.group_fills, self.groups, self.next.clone()?);
        std::mem::replace(&mut self.next, next)
    }
}

impl ArrangementSet {
    fn new(group_fills: &[Vec<BitSet>], groups: &[usize], num: usize) -> Self {
        let mut mask = BitSet::empty(group_fills[0][0].bits());

        for group in groups.iter().map(|g| group_fills[*g].last().unwrap()) {
            mask += group;
        }

        let mut group_set = BitSet::empty(mask.bits());
        group_set.extend(groups.iter().copied());

        Self {
            mask,
            groups: group_set,
            arrangements: CombinationsIter::new(group_fills, groups, num).collect(),
        }
    }
}

fn group_fills(groups: &[BitSet]) -> Vec<Vec<BitSet>> {
    groups
        .iter()
        .map(|group| {
            let mut fills = Vec::with_capacity(group.count_ones() + 1);
            fills.push(BitSet::empty(group.bits()));
            fills.extend(
                group
                    .iter_ones()
                    .scan(BitSet::empty(group.bits()), |state, item| {
                        state.set_to_one(item);

                        Some(state.clone())
                    }),
            );
            fills
        })
        .collect()
}

impl<G: Graph> Board<G> {
    /// Assigns a group id to each empty cell so that empty cells with the same id are constrained
    /// by the same set of hints.
    fn cell_groups(&self) -> Vec<Option<usize>> {
        let mut mapping: Vec<usize> = Vec::with_capacity(self.grid.len());
        let mut is_mapped: Vec<bool> = Vec::with_capacity(self.grid.len());
        let mut group_ids: Vec<usize> = vec![0; self.grid.len()];

        let mut max_group = 1;

        mapping.push(0);
        is_mapped.push(false);

        for i in self
            .grid
            .iter()
            .positions(|c| matches!(c, Hint { empties: 1.., .. }))
        {
            for j in self.neighbors(i) {
                if self.grid[j] != Empty {
                    continue;
                }

                let id = group_ids[j];

                if !is_mapped[id] {
                    mapping[id] = max_group;
                    is_mapped[id] = true;
                    max_group += 1;

                    mapping.push(0);
                    is_mapped.push(false);
                }
            }

            for j in self.neighbors(i) {
                if self.grid[j] != Empty {
                    continue;
                }

                let id = group_ids[j];

                group_ids[j] = mapping[id];
                is_mapped[id] = false;
            }
        }

        max_group = 0;

        let mut out = vec![None; self.grid.len()];

        for (i, id) in group_ids.iter().copied().enumerate() {
            if self.grid[i] != Empty || id == 0 {
                continue;
            }

            if !is_mapped[id] {
                mapping[id] = max_group;
                is_mapped[id] = true;
                max_group += 1;
            }

            out[i] = Some(mapping[id]);
        }

        out
    }

    pub fn all_empties(&self) -> BitSet {
        let mut out = BitSet::empty(self.num_tiles());

        for (i, tile) in self.grid.iter().enumerate() {
            if *tile == Empty {
                out.set_to_one(i);
            }
        }

        out
    }

    fn initial_solutionset(&self) -> MineArrangements {
        let cell_groups = self.cell_groups();
        let mut groups = Vec::with_capacity(64);

        for (i, group) in cell_groups.iter().enumerate() {
            let Some(group) = group else { continue };

            if *group >= groups.len() {
                groups.resize_with(group + 1, || BitSet::empty(self.num_tiles()));
            }

            groups[*group].set_to_one(i);
        }

        let mut sub_solutions: Vec<ArrangementSet> = Vec::new();
        let group_fills = group_fills(&groups);

        for (i, tile) in self.grid.iter().enumerate() {
            if let Hint {
                remaining_mines: mines @ 1..,
                ..
            } = tile
            {
                let mut groups = Vec::with_capacity(32);
                groups.extend(self.neighbors(i).filter_map(|j| cell_groups[j]));
                groups.sort_unstable();
                groups.dedup();

                sub_solutions.push(ArrangementSet::new(&group_fills, &groups, *mines as usize));
            }
        }

        MineArrangements {
            groups,
            sub_arrangements: sub_solutions,
            mask: self.all_empties(),
            remaining_mines: self.remaining_mines(),
            num_tiles: self.num_tiles(),
        }
    }

    pub fn solutionset(&self) -> MineArrangements {
        let mut out = self.initial_solutionset();
        out.merge_all_subsolutions();
        out.filter_summaries();
        out
    }
}

impl ArrangementSet {
    fn remove_mask(&mut self, mask: &BitSet) {
        for arrangement in &mut self.arrangements {
            *arrangement -= mask;
        }

        self.mask -= mask;
    }

    #[must_use]
    fn try_merge_equal_value(&mut self, other: &mut Self, overlap: &mut BitSet) -> Option<()> {
        let overlap_arrangement = self
            .arrangements
            .iter()
            .map(|arr| arr & &*overlap)
            .all_equal_value()
            .ok()?;

        other
            .arrangements
            .retain(|arrangement| overlap_arrangement.equal_on_mask(arrangement, overlap));
        other.remove_mask(overlap);

        if self.arrangements.len() <= 1 || other.arrangements.len() <= 1 {
            *overlap = BitSet::empty(self.mask.bits());
            return None;
        }

        Some(())
    }

    #[allow(clippy::result_large_err)]
    fn try_merge(&mut self, mut other: Self) -> Result<(), Self> {
        let merged_mask = &self.mask | &other.mask;
        let mut overlap = &self.mask & &other.mask;

        if self
            .try_merge_equal_value(&mut other, &mut overlap)
            .or_else(|| other.try_merge_equal_value(self, &mut overlap))
            .is_some()
        {
            if !self.mask.any() {
                std::mem::swap(self, &mut other);
                return Ok(());
            }
            if other.mask.any() {
                return Err(other);
            }
            return Ok(());
        }

        let mut new_arrangements =
            Vec::with_capacity(self.arrangements.len().max(other.arrangements.len()));

        for arr1 in &self.arrangements {
            for arr2 in &other.arrangements {
                if arr1.equal_on_mask(arr2, &overlap) {
                    new_arrangements.push(arr1 | arr2);
                }
            }
        }

        self.mask = merged_mask;
        self.arrangements = new_arrangements;

        self.groups += other.groups;

        Ok(())
    }

    fn solved_safe(&self, groups: &[BitSet]) -> BitSet {
        let mut out = self.mask.clone();

        for arrangement in &self.arrangements {
            out -= arrangement;
        }

        for group in self.groups.iter_ones().map(|group| &groups[group]) {
            if !group.is_subset_of(&out) {
                out -= group;
            }
        }

        out
    }

    fn solved_mines(&self, groups: &[BitSet]) -> BitSet {
        let mut out = self.mask.clone();

        for arrangement in &self.arrangements {
            out &= arrangement;
        }

        for group in self.groups.iter_ones().map(|group| &groups[group]) {
            if !group.is_subset_of(&out) {
                out -= group;
            }
        }

        out
    }

    fn summarize(&self) -> BitSet {
        let mut out = BitSet::empty(self.mask.bits());

        for arr in &self.arrangements {
            out.set_to_one(arr.count_ones());
        }

        out
    }

    fn retain_with_summary(&mut self, summary: &BitSet) {
        self.arrangements
            .retain(|arrangement| summary.get(arrangement.count_ones()))
    }

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
        let mut counts = vec![vec![0; self.mask.count_ones()]; self.mask.count_ones() + 1];

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
    fn merge_all_subsolutions(&mut self) {
        let mut i = 0;

        while i < self.sub_arrangements.len() {
            let pos = self
                .sub_arrangements
                .iter()
                .enumerate()
                .skip(i + 1)
                .map(|(j, sol)| {
                    (
                        sol.mask.count_overlap_ones(&self.sub_arrangements[i].mask),
                        j,
                    )
                })
                .max_by_key(|(a, _)| *a);

            if let Some((1.., pos)) = pos {
                let tmp = self.sub_arrangements.swap_remove(pos);

                if let Err(tmp) = self.sub_arrangements[i].try_merge(tmp) {
                    self.sub_arrangements.push(tmp);
                }
            } else {
                i += 1;
            }
        }
    }

    pub fn solved(&self) -> (BitSet, BitSet) {
        let mut safe = BitSet::empty(self.mask.bits());
        let mut mines = BitSet::empty(self.mask.bits());

        for sub_solution in &self.sub_arrangements {
            safe += sub_solution.solved_safe(&self.groups);
            mines += sub_solution.solved_mines(&self.groups);
        }

        let uncontrained_empties = self.uncontrained_empties();

        if mines.count_ones() == self.remaining_mines {
            safe += &uncontrained_empties;
        } else if self.mask.count_ones() - safe.count_ones() == self.remaining_mines {
            mines += &uncontrained_empties;
        }

        (safe, mines)
    }

    fn uncontrained_empties(&self) -> BitSet {
        self.sub_arrangements
            .iter()
            .fold(self.mask.clone(), |mut out, arr| {
                out -= &arr.mask;
                out
            })
    }

    fn filter_summaries(&mut self) {
        let unconstrained_empties = self.uncontrained_empties().count_ones();

        let valid_range =
            self.remaining_mines.saturating_sub(unconstrained_empties)..=self.remaining_mines;
        let mut valid_summaries =
            vec![BitSet::empty(self.mask.bits()); self.sub_arrangements.len()];

        self.sub_arrangements
            .iter()
            .map(|arr| arr.summarize().iter_ones().collect_vec())
            .multi_cartesian_product()
            .for_each(|vec| {
                if !valid_range.contains(&vec.iter().sum::<usize>()) {
                    return;
                }

                for (summary, mine_count) in valid_summaries.iter_mut().zip(&vec) {
                    summary.set_to_one(*mine_count);
                }
            });

        for (arrangement, summary) in self.sub_arrangements.iter_mut().zip(valid_summaries) {
            arrangement.retain_with_summary(&summary);
        }
    }

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

    pub fn tile_safes(&self) -> Vec<Natural> {
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

            unconstrained_count += total_solutions * Natural::from(unconstrained_mines)
                / Natural::from(num_unconstrained);
        }

        for tile in self.uncontrained_empties().iter_ones() {
            out[tile] = unconstrained_count.clone();
        }

        out
    }
}
