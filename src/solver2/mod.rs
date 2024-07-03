use std::collections::HashSet;

use itertools::Itertools;

mod bitset;

use bitset::BitSet;

use crate::{Game, Graph};

#[derive(Clone, Debug)]
struct MineArrangementSet {
    /// The set of tiles that this MineArrangementList covers
    mask: BitSet,

    /// A set of non-overlapping regions where a particular number of mines may exist
    regions: HashSet<BitSet>,

    /// The list of possible arrangements of mines
    arrangements: Vec<BitSet>,
}

#[derive(Clone, Debug)]
pub struct MineArrangements {
    total_mines: usize,
    old_safe: BitSet,
    sub_arrangements: Vec<MineArrangementSet>,
}

fn split_arrangement(
    mask1: &BitSet,
    mask2: &BitSet,
    mut arrangement: BitSet,
    out: &mut Vec<BitSet>,
) {
    let mask1_size = mask1.count_ones();
    let mines = &arrangement & (mask1 + mask2);
    let mine_count = mines.count_ones();

    arrangement -= mines;

    let mask1_starting_mines = mine_count.min(mask1_size);
    let mask2_starting_mines = mine_count.saturating_sub(mask1_size);

    for i in mask1.iter_ones().take(mask1_starting_mines) {
        arrangement.set_to_one(i);
    }

    for i in mask2.iter_ones().take(mask2_starting_mines) {
        arrangement.set_to_one(i);
    }

    for (i, j) in mask1
        .iter_ones()
        .rev()
        .skip(mask1_size - mask1_starting_mines)
        .zip(mask2.iter_ones().skip(mask2_starting_mines))
    {
        out.push(arrangement.clone());
        arrangement.set_to_zero(i);
        arrangement.set_to_one(j);
    }

    out.push(arrangement.clone());
}

impl MineArrangementSet {
    fn empty(grid_size: usize) -> Self {
        Self {
            mask: BitSet::zeros(grid_size),
            regions: HashSet::new(),
            arrangements: Vec::new(),
        }
    }

    fn from_constraint(mask: BitSet, mine_count: usize) -> Self {
        let mut arrangement = BitSet::zeros(mask.len());

        for i in mask.iter_ones().take(mine_count) {
            arrangement.set_to_one(i);
        }

        let mut regions = HashSet::new();
        regions.insert(mask.clone());

        Self {
            regions,
            mask,
            arrangements: vec![arrangement],
        }
    }

    #[must_use]
    fn split_region(&mut self, new_region: BitSet) -> Option<()> {
        let mut split_region = self
            .regions
            .iter()
            .find(|region| new_region.is_subset_of(region))?
            .clone();

        self.regions.remove(&split_region);
        split_region -= &new_region;

        for arrangement in std::mem::take(&mut self.arrangements) {
            split_arrangement(
                &split_region,
                &new_region,
                arrangement,
                &mut self.arrangements,
            );
        }

        self.regions.insert(split_region);
        self.regions.insert(new_region);

        Some(())
    }

    fn merge_solved_regions(&mut self) {
        if self.arrangements.len() != 1 {
            return;
        }

        let mine_tiles = self.mine_tiles();
        let safe_tiles = self.safe_tiles();

        self.regions.retain(|region| {
            !region.is_subset_of(&mine_tiles) && !region.is_subset_of(&safe_tiles)
        });
        self.regions.insert(mine_tiles);
        self.regions.insert(safe_tiles);
    }

    fn split_regions_for_merge(&mut self, other: &mut Self) {
        let mut new_regions_1 = Vec::new();
        let mut new_regions_2 = Vec::new();

        for region1 in &self.regions {
            for region2 in &other.regions {
                if !region1.overlaps_with(region2) {
                    continue;
                }

                let new_region = region1 & region2;

                if new_region != *region1 {
                    new_regions_1.push(new_region.clone());
                }
                if new_region != *region2 {
                    new_regions_2.push(new_region);
                }
            }
        }

        for region in new_regions_1 {
            self.split_region(region).unwrap();
        }
        for region in new_regions_2 {
            other.split_region(region).unwrap();
        }
    }

    fn remove_mask(&mut self, mask: &BitSet) {
        self.regions.retain(|region| !region.is_subset_of(mask));

        for arrangement in &mut self.arrangements {
            *arrangement -= mask;
        }

        self.mask -= mask;
    }

    #[allow(clippy::result_large_err)]
    fn try_merge(&mut self, mut other: Self) -> Result<(), Self> {
        let merged_mask = &self.mask + &other.mask;
        let mut overlap = &self.mask & &other.mask;

        self.split_regions_for_merge(&mut other);

        let arrangements_equal_value = |arrs: &[BitSet]| {
            if arrs.len() <= 1 {
                return None;
            }
            arrs.iter().map(|arr| arr & &overlap).all_equal_value().ok()
        };

        let overlap_arrangement = arrangements_equal_value(&self.arrangements).or_else(|| {
            arrangements_equal_value(&other.arrangements)
                .inspect(|_| std::mem::swap(self, &mut other))
        });

        if let Some(overlap_arrangement) = overlap_arrangement {
            other
                .arrangements
                .retain(|arrangement| overlap_arrangement.equal_on_mask(arrangement, &overlap));
            other.remove_mask(&overlap);

            if !other.mask.any() {
                return Ok(());
            }
            if other.arrangements.len() != 1 {
                return Err(other);
            }
            overlap = BitSet::zeros(self.mask.len());
        }

        let mut new_arrangements = Vec::new();

        for arr1 in &self.arrangements {
            for arr2 in &other.arrangements {
                if arr1.equal_on_mask(arr2, &overlap) {
                    new_arrangements.push(arr1 + arr2);
                }
            }
        }

        self.regions.extend(other.regions);
        self.mask = merged_mask;
        self.arrangements = new_arrangements;

        self.merge_solved_regions();

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

        for region in &self.regions {
            if !region.is_subset_of(&out) {
                out -= region;
            }
        }

        out
    }

    fn mine_tiles(&self) -> BitSet {
        let mut out = self.mask.clone();

        for arrangement in &self.arrangements {
            out &= arrangement;
        }

        for region in &self.regions {
            if !region.is_subset_of(&out) {
                out -= region;
            }
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

        // print!("{pos} ");
        let num_mines = game.explore_tile(pos)?;
        // println!("{num_mines}");
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
