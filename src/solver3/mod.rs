use itertools::Itertools;

mod bitset;
mod combinations;
mod tree_solver;

use bitset::BitSet;
use combinations::CombinationsIter;

use crate::{Game, Graph};

#[derive(Clone, Debug)]
struct MineArrangementSet {
    /// The set of tiles that this MineArrangementList covers
    mask: BitSet,

    /// The list of possible arrangements of mines
    arrangements: Vec<BitSet>,
}

#[derive(Clone, Debug)]
pub struct MineArrangements {
    total_mines: usize,
    old_safe: BitSet,
    sub_arrangements: Vec<MineArrangementSet>,
}

impl MineArrangementSet {
    fn empty(grid_size: usize) -> Self {
        Self {
            mask: BitSet::zeros(grid_size),
            arrangements: Vec::new(),
        }
    }

    fn from_constraint(mask: BitSet, mine_count: usize) -> Self {
        Self {
            arrangements: CombinationsIter::new(mask.clone(), mine_count).collect(),
            mask,
        }
    }

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

        Ok(())
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

    fn safe_tiles(&self) -> BitSet {
        let mut out = self.mask.clone();

        for arrangement in &self.arrangements {
            out -= arrangement;
        }

        out
    }

    fn mine_tiles(&self) -> BitSet {
        let mut out = self.mask.clone();

        for arrangement in &self.arrangements {
            out &= arrangement;
        }

        out
    }
}

impl MineArrangements {
    pub fn new(size: usize, total_mines: usize) -> Self {
        Self {
            total_mines,
            old_safe: BitSet::zeros(size),
            sub_arrangements: Vec::new(),
        }
    }

    pub fn from_game(game: &(impl Game + Clone + Eq)) -> Self {
        Self::new(game.num_tiles(), game.num_mines())
    }

    pub fn num_tiles(&self) -> usize {
        self.old_safe.len()
    }

    fn add_arrangement_set(&mut self, mut new_arrangement_set: MineArrangementSet) {
        while let Some(i) = self
            .sub_arrangements
            .iter()
            .position_max_by_key(|arr| arr.mask.count_overlap_ones(&new_arrangement_set.mask))
        {
            if !self.sub_arrangements[i]
                .mask
                .overlaps_with(&new_arrangement_set.mask)
            {
                break;
            }

            let arr = self.sub_arrangements.swap_remove(i);
            if let Err(arr) = new_arrangement_set.try_merge(arr) {
                self.sub_arrangements.push(arr);
            }
        }

        self.sub_arrangements.push(new_arrangement_set);
    }

    pub fn add_constraint(&mut self, mask: BitSet, num_mines: usize) {
        self.add_arrangement_set(MineArrangementSet::from_constraint(mask, num_mines))
    }

    #[must_use]
    pub fn add_constraint_with_game(
        &mut self,
        pos: usize,
        game: &mut (impl Game + Clone + Eq),
    ) -> Option<()> {
        let mut mask1 = BitSet::zeros(self.num_tiles());
        let mut mask2 = BitSet::zeros(self.num_tiles());

        self.old_safe.set_to_one(pos);

        let num_mines = game.explore_tile(pos)?;
        for pos2 in game.graph().neighbors(pos) {
            mask1.set_to_one(pos2);
        }
        mask2.set_to_one(pos);

        self.add_constraint(mask1, num_mines as usize);
        self.add_constraint(mask2, 0);

        Some(())
    }

    pub fn play_game(&mut self, game: &mut (impl Game + Clone + Eq)) {
        assert_eq!(self.num_tiles(), game.num_tiles());

        loop {
            let new_safe = self.new_safe_tiles();

            if !new_safe.any() {
                break;
            }

            for i in new_safe.iter_ones() {
                self.add_constraint_with_game(i, game).unwrap();
            }
        }
    }

    fn filter_summaries(&mut self) {
        let unconstrained_empties = self
            .sub_arrangements
            .iter()
            .fold(BitSet::ones(self.num_tiles()), |mut out, arr| {
                out -= &arr.mask;
                out
            })
            .count_ones();

        let valid_range = self.total_mines.saturating_sub(unconstrained_empties)..=self.total_mines;
        let mut valid_summaries =
            vec![BitSet::zeros(self.num_tiles()); self.sub_arrangements.len()];

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

    pub fn safe_tiles(&self) -> BitSet {
        self.sub_arrangements
            .iter()
            .fold(BitSet::zeros(self.num_tiles()), |mut sum, arr| {
                sum += arr.safe_tiles();
                sum
            })
    }

    pub fn mine_tiles(&self) -> BitSet {
        self.sub_arrangements
            .iter()
            .fold(BitSet::zeros(self.num_tiles()), |mut sum, arr| {
                sum += arr.mine_tiles();
                sum
            })
    }

    pub fn new_safe_tiles(&mut self) -> BitSet {
        let mut out = self.safe_tiles() - &self.old_safe;

        if out.any() {
            return out;
        }

        self.filter_summaries();
        out = self.safe_tiles() - &self.old_safe;

        out
    }
}
