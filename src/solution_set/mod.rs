use itertools::Itertools;

use crate::{bitset::BitSet, board::*, game::*};

mod combinations_iter;
// mod incremental;
pub mod solution_counting;

use combinations_iter::CombinationsIter;

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

impl ArrangementSet {
    fn new(groups: &[BitSet], group_set: &BitSet, num: usize) -> Self {
        let mut mask = BitSet::empty(group_set.bits());

        for group in group_set.iter_ones().map(|g| &groups[g]) {
            mask += group;
        }

        Self {
            mask,
            groups: group_set.clone(),
            arrangements: CombinationsIter::new(groups, group_set, num).collect(),
        }
    }
}

impl<G: Graph> Board<G> {
    /// Assigns a group id to each empty cell so that empty cells with the same id are constrained
    /// by the same set of hints.
    fn cell_groups(&self) -> Vec<Option<usize>> {
        let mut mapping: Vec<(usize, usize)> = vec![(0, 0); self.grid.len()];
        let mut group_ids: Vec<usize> = vec![0; self.grid.len()];

        let mut max_group = 1;

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

                if mapping[id].1 < i {
                    if id >= mapping.len() {
                        mapping.resize(id + 1, (0, 0));
                    }
                    mapping[id] = (max_group, i);
                    group_ids[j] = max_group;
                    max_group += 1;
                } else {
                    group_ids[j] = mapping[id].0;
                }
            }
        }

        max_group = 0;

        let mut out = vec![None; self.grid.len()];

        for (i, id) in group_ids.iter().copied().enumerate() {
            if self.grid[i] != Empty || id == 0 {
                continue;
            }

            if id >= mapping.len() {
                mapping.resize(id + 1, (0, 0));
            }

            if mapping[id].1 != usize::MAX {
                mapping[id] = (max_group, usize::MAX);
                max_group += 1;
            }

            out[i] = Some(mapping[id].0);
        }

        out
    }
}

fn bitset_groups(cell_groups: &[Option<usize>]) -> Vec<BitSet> {
    let mut groups = Vec::with_capacity(64);

    for (i, group) in cell_groups.iter().enumerate() {
        let Some(group) = group else { continue };

        if *group >= groups.len() {
            groups.resize_with(group + 1, || BitSet::empty(cell_groups.len()));
        }

        groups[*group].set_to_one(i);
    }

    groups
}

impl<G: Graph> Board<G> {
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

        for (i, tile) in self.grid.iter().enumerate() {
            if let Hint {
                remaining_mines,
                empties: _empties @ 1..,
                ..
            } = tile
            {
                let group_set = BitSet::from_iter(
                    self.neighbors(i).filter_map(|j| cell_groups[j]),
                    self.num_tiles(),
                );

                sub_solutions.push(ArrangementSet::new(
                    &groups,
                    &group_set,
                    *remaining_mines as usize,
                ));
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
    fn remove(&mut self, other: &Self) {
        for arrangement in &mut self.arrangements {
            *arrangement -= &other.mask;
        }

        self.groups -= &other.groups;
        self.mask -= &other.mask;
    }

    #[must_use]
    fn try_merge_equal_value(&mut self, other: &mut Self, overlap: &mut BitSet) -> Option<()> {
        let overlap_arrangement = self.arrangements.first()? & &*overlap;

        if !self.arrangements[1..]
            .iter()
            .all(|arr| arr.equal_on_mask(&overlap_arrangement, overlap))
        {
            return None;
        }

        other
            .arrangements
            .retain(|arrangement| overlap_arrangement.equal_on_mask(arrangement, overlap));
        other.remove(self);

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
                        j,
                        sol.mask.count_overlap_ones(&self.sub_arrangements[i].mask),
                    )
                })
                .max_by_key(|(_, a)| *a);

            if let Some((pos, 1..)) = pos {
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
            .map(ArrangementSet::summarize)
            .collect_vec()
            .iter()
            .map(BitSet::iter_ones)
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
