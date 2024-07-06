use super::*;

fn split_arrangement(
    out: &mut Vec<BitSet>,
    arrangement: &BitSet,
    group_splits: &[BitSet],
    groups: &[BitSet],
    group_set: &BitSet,
) {
    let things: Vec<Vec<BitSet>> = group_set
        .iter_ones()
        .map(|i| {
            CombinationsIter::new(
                groups,
                &group_splits[i],
                arrangement.count_overlap_ones(&groups[i]),
            )
            .collect()
        })
        .collect();

    out.extend(things.iter().multi_cartesian_product().map(|iter| {
        let mut arrangement = BitSet::empty(arrangement.bits());

        for group in iter {
            arrangement |= group;
        }

        arrangement
    }));
}

fn group_splits(old_groups: &[BitSet], new_groups: &[BitSet]) -> Vec<BitSet> {
    old_groups
        .iter()
        .map(|old_group| {
            BitSet::from_iter(
                new_groups.iter().positions(|x| x.is_subset_of(old_group)),
                old_groups[0].bits(),
            )
        })
        .collect()
}

impl ArrangementSet {
    fn with_groups(&self, new_groups: &[BitSet], group_splits: &[BitSet]) -> Self {
        let new_group_set = BitSet::from_iter(
            new_groups.iter().positions(|x| x.is_subset_of(&self.mask)),
            self.mask.bits(),
        );

        let mut arrangements = Vec::new();

        for arrangement in &self.arrangements {
            split_arrangement(
                &mut arrangements,
                arrangement,
                group_splits,
                new_groups,
                &self.groups,
            );
        }

        Self {
            mask: self.mask.clone(),
            groups: new_group_set,
            arrangements,
        }
    }
}

impl MineArrangements {
    pub fn increment(&self, board: &Board<impl Graph>) -> Self {
        let cell_groups = board.cell_groups();
        let groups = bitset_groups(&cell_groups);

        let splits = group_splits(&self.groups, &groups);
        let mask = board.all_empties();

        let mut sub_arrangements = self
            .sub_arrangements
            .iter()
            .map(|arr| arr.with_groups(&groups, &splits))
            .collect_vec();

        sub_arrangements.extend(
            mask.iter_ones()
                .filter_map(|i| board.new_arrangement_set(i, &groups, &cell_groups)),
        );

        let mut out = MineArrangements {
            groups: groups.clone(),
            sub_arrangements,
            mask: board.all_empties(),
            remaining_mines: board.remaining_mines(),
            num_tiles: board.num_tiles(),
        };

        println!("{out:#?}");

        out.merge_all_subsolutions();
        out.filter_summaries();
        out
    }
}
