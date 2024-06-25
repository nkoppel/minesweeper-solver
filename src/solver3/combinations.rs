use super::*;

fn next_combination(mask: &BitSet, mut combination: BitSet) -> Option<BitSet> {
    let first_zeroed = combination.iter_ones().next()?;

    combination.set_to_zero(first_zeroed);

    for (i1, i2) in mask
        .iter_ones()
        .zip(mask.iter_ones().skip_while(|&i| i <= first_zeroed))
    {
        if !combination.get(i2) {
            combination.set_to_one(i2);
            return Some(combination);
        }

        combination.set_to_one(i1);
        combination.set_to_zero(i2);
    }

    None
}

pub struct CombinationsIter {
    combination: Option<BitSet>,
    mask: BitSet,
}

impl CombinationsIter {
    pub fn new(mask: BitSet, num: usize) -> Self {
        let mut combination = BitSet::zeros(mask.len());

        for i in mask.iter_ones().take(num) {
            combination.set_to_one(i);
        }

        Self {
            mask,
            combination: Some(combination),
        }
    }
}

impl Iterator for CombinationsIter {
    type Item = BitSet;

    fn next(&mut self) -> Option<BitSet> {
        let combination = self.combination.take()?;
        self.combination = next_combination(&self.mask, combination.clone());

        Some(combination)
    }
}
