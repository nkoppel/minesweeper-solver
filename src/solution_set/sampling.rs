use malachite::{
    base::{num::random::random_primitive_ints, random::Seed},
    natural::random::get_random_natural_less_than,
    Natural,
};
use rand::{random_range, rng, seq::SliceRandom};
use smallvec::SmallVec;

use super::{solution_counting::get_arrangement_count, *};

fn random_natural_less_than(bound: &Natural) -> Natural {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    let seed = Seed { bytes };
    let mut primitives = random_primitive_ints(seed);
    get_random_natural_less_than(&mut primitives, bound)
}

fn select_weighted_random_natural<'a>(
    naturals: impl IntoIterator<Item = &'a Natural> + Clone,
) -> usize {
    let sum = naturals.clone().into_iter().sum();
    let mut random = random_natural_less_than(&sum);

    for (i, val) in naturals.into_iter().enumerate() {
        if val > &random {
            return i;
        }
        random -= val;
    }

    unreachable!()
}

fn random_combination(mask: &BitSet, n: usize) -> BitSet {
    let mut indices: SmallVec<[usize; 16]> = mask.iter_ones().collect();
    let (shuffled, _) = indices.partial_shuffle(&mut rng(), n);
    BitSet::from_iter(shuffled.iter().copied(), mask.bits())
}

fn select_weighted_random<T>(iter: impl IntoIterator<Item = (T, u64)> + Clone) -> Option<T> {
    let sum = iter.clone().into_iter().map(|(_, c)| c).sum();
    let mut random = random_range(0..sum);

    for (val, count) in iter.into_iter() {
        if count > random {
            return Some(val);
        }
        random -= count;
    }

    None
}

impl MineArrangements {
    pub fn sample_arrangement(&self) -> BitSet {
        let solution_counts = self.solution_counts();
        let group_sizes = self.groups.iter().map(BitSet::count_ones).collect_vec();
        let i = select_weighted_random_natural(solution_counts.iter().map(|(count, _)| count));
        let counts = &solution_counts[i].1;

        let unconstrained_mines =
            self.remaining_mines - counts.iter().map(|(mines, _)| mines).sum::<usize>();
        let mut out = random_combination(&self.uncontrained_empties(), unconstrained_mines);

        for ((num_mines, _), arrangement) in counts.iter().zip(&self.sub_arrangements) {
            let arr = select_weighted_random(
                arrangement
                    .arrangements
                    .iter()
                    .filter(|arr| arr.count_ones() == *num_mines)
                    .map(|arr| {
                        (
                            arr,
                            get_arrangement_count(
                                arr,
                                &arrangement.groups,
                                &self.groups,
                                &group_sizes,
                            ),
                        )
                    }),
            )
            .unwrap();

            for group in arrangement.groups.iter_ones() {
                let group_mask = &self.groups[group];
                let num_mines = group_mask.count_overlap_ones(arr);
                out += random_combination(group_mask, num_mines);
            }
        }

        out
    }

    pub fn sample_arrangement_with_board(&self, board: &Board<impl Graph>) -> BitSet {
        assert_eq!(self.num_tiles, board.num_tiles());

        let out = board.known_mines() + self.sample_arrangement();
        assert_eq!(out.count_ones(), board.num_mines);
        out
    }

    pub fn sample_game_with_board<G: Graph>(&self, board: &Board<G>) -> InternalGame<G> {
        let arrangement = self.sample_arrangement_with_board(board);
        InternalGame::from_grid(arrangement, board.graph.clone())
    }

    pub fn sample_game<G: Graph>(&self, game: &InternalGame<G>) -> InternalGame<G> {
        assert_eq!(self.num_tiles, game.num_tiles());

        let mut out = game.clone();
        let arrangement = self.sample_arrangement();

        if let Some(grid) = out.grid.as_mut() {
            *grid -= &self.mask;
            *grid += arrangement;
        } else {
            out.grid = Some(arrangement);
        }

        assert_eq!(out.grid.as_ref().unwrap().count_ones(), game.num_mines());

        out
    }
}
