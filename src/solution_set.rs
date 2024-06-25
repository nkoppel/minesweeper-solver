use itertools::Itertools;

use crate::{bitset::BitSet, board::*, game::*};

#[derive(Clone, Debug)]
struct ArrangementSet {
    mask: BitSet,
    groups: Vec<usize>,
    arrangements: Vec<BitSet>,
}

#[derive(Clone, Debug)]
pub struct MineArrangements {
    groups: Vec<BitSet>,
    sub_arrangements: Vec<ArrangementSet>,
    mask: BitSet,
    remaining_mines: usize,
}

struct CombinationsIter<'a> {
    group_fills: &'a [Vec<BitSet>],
    groups: &'a [usize],
    next: Option<BitSet>,
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
        let mut next = BitSet::zeros(group_fills[0][0].len());
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
        let next = std::mem::take(&mut self.next)?;
        self.next = next_combination(self.group_fills, self.groups, next.clone());
        Some(next)
    }
}

impl ArrangementSet {
    fn new(group_fills: &[Vec<BitSet>], groups: &[usize], num: usize) -> Self {
        let mut mask = BitSet::zeros(group_fills[0][0].len());

        for group in groups.iter().map(|g| group_fills[*g].last().unwrap()) {
            mask += group;
        }

        Self {
            mask,
            groups: groups.to_vec(),
            arrangements: CombinationsIter::new(group_fills, groups, num).collect(),
        }
    }
}

fn group_fills(groups: &[BitSet]) -> Vec<Vec<BitSet>> {
    groups
        .iter()
        .map(|group| {
            let mut fills = vec![BitSet::zeros(group.len())];
            fills.extend(
                group
                    .iter_ones()
                    .scan(BitSet::zeros(group.len()), |state, item| {
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
        let mut out = BitSet::zeros(self.num_tiles());

        for (i, tile) in self.grid.iter().enumerate() {
            if *tile == Empty {
                out.set_to_one(i);
            }
        }

        out
    }

    fn initial_solutionset(&self) -> MineArrangements {
        let cell_groups = self.cell_groups();
        let mut groups = Vec::new();

        for (i, group) in cell_groups.iter().enumerate() {
            let Some(group) = group else { continue };

            if *group >= groups.len() {
                groups.resize_with(group + 1, || BitSet::zeros(self.num_tiles()));
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
                let mut groups = Vec::new();
                for j in self.neighbors(i) {
                    if let Some(group) = cell_groups[j] {
                        groups.push(group);
                    }
                }
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
            *overlap = BitSet::zeros(overlap.len());
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

        let mut new_arrangements = Vec::new();

        for arr1 in &self.arrangements {
            for arr2 in &other.arrangements {
                if arr1.equal_on_mask(arr2, &overlap) {
                    new_arrangements.push(arr1 | arr2);
                }
            }
        }

        self.mask = merged_mask;
        self.arrangements = new_arrangements;

        self.groups.append(&mut other.groups);
        self.groups.sort_unstable();
        self.groups.dedup();

        Ok(())
    }

    fn solved_safe(&self, groups: &[BitSet]) -> BitSet {
        let mut out = self.mask.clone();

        for arrangement in &self.arrangements {
            out -= arrangement;
        }

        for group in self.groups.iter().map(|group| &groups[*group]) {
            if !group.is_subset_of(&out) {
                out -= group;
            }
        }

        out
    }

    fn solved_mines(&self, groups: &[BitSet]) -> BitSet {
        let mut out = BitSet::ones(self.mask.len());

        for arrangement in &self.arrangements {
            out &= arrangement;
        }

        for group in self.groups.iter().map(|group| &groups[*group]) {
            if !group.is_subset_of(&out) {
                out -= group;
            }
        }

        out
    }

    fn summarize(&self) -> BitSet {
        let mut out = BitSet::zeros(self.mask.len());

        for arr in &self.arrangements {
            out.set_to_one(arr.count_ones());
        }

        out
    }

    fn retain_with_summary(&mut self, summary: &BitSet) {
        self.arrangements
            .retain(|arrangement| summary.get(arrangement.count_ones()))
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
        let mut safe = BitSet::zeros(self.mask.len());
        let mut mines = BitSet::zeros(self.mask.len());

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
        let mut valid_summaries = vec![BitSet::zeros(self.mask.len()); self.sub_arrangements.len()];

        self.sub_arrangements
            .iter()
            .map(|arr| arr.summarize().into_iter_ones())
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
}
