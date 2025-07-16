use crate::bitset::BitSet;

pub struct CombinationsIter<'a> {
    groups: &'a [BitSet],
    group_set: &'a BitSet,
    next: Option<BitSet>,
}

fn fill_front(
    groups: &[BitSet],
    group_set: &BitSet,
    combination: &mut BitSet,
    mut num_ones: usize,
) {
    for group in group_set.iter_ones().map(|g| &groups[g]) {
        let group_size = group.count_ones();

        if group_size >= num_ones {
            *combination += group.first_n_ones(num_ones);
            break;
        }

        *combination += group;
        num_ones -= group_size;
    }
}

fn next_combination(
    groups: &[BitSet],
    group_set: &BitSet,
    mut combination: BitSet,
) -> Option<BitSet> {
    let mut group_iter = group_set.iter_ones();

    let start_group = loop {
        let group = &groups[group_iter.next()?];

        if combination.overlaps_with(group) {
            break group;
        }
    };

    let mut zeroed_count = start_group.count_overlap_ones(&combination);
    combination -= start_group;

    for group_idx in group_iter {
        let group = &groups[group_idx];

        if !group.is_subset_of(&combination) {
            let ones = combination.count_overlap_ones(group);
            combination.set_to_one(group.iter_ones().nth(ones).unwrap());
            fill_front(groups, group_set, &mut combination, zeroed_count - 1);
            return Some(combination);
        }

        zeroed_count += combination.count_overlap_ones(group);
        combination -= group;
    }

    None
}

impl<'a> CombinationsIter<'a> {
    pub fn new(groups: &'a [BitSet], group_set: &'a BitSet, num: usize) -> Self {
        let num_tiles: usize = group_set
            .iter_ones()
            .map(|group| groups[group].count_ones())
            .sum();
        if num > num_tiles {
            return Self {
                groups,
                group_set,
                next: None,
            };
        }

        let mut next = BitSet::empty(group_set.bits());
        fill_front(groups, group_set, &mut next, num);

        Self {
            groups,
            group_set,
            next: Some(next),
        }
    }
}

impl Iterator for CombinationsIter<'_> {
    type Item = BitSet;

    fn next(&mut self) -> Option<Self::Item> {
        let next = next_combination(self.groups, self.group_set, self.next.clone()?);
        std::mem::replace(&mut self.next, next)
    }
}
