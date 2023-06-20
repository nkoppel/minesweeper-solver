use crate::game::*;
use crate::solver::{csp::*, *};

use itertools::Itertools;
use smallvec::*;

use super::solutionset::SolutionSet;

impl<G: Game> Solver<G> {
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
            self.game.for_each_neighbor(i, |j| {
                if self.grid[j] != Empty {
                    return;
                }

                let id = group_ids[j];

                if !is_mapped[id] {
                    mapping[id] = max_group;
                    is_mapped[id] = true;
                    max_group += 1;

                    mapping.push(0);
                    is_mapped.push(false);
                }
            });

            self.game.for_each_neighbor(i, |j| {
                if self.grid[j] != Empty {
                    return;
                }

                let id = group_ids[j];

                group_ids[j] = mapping[id];
                is_mapped[id] = false;
            });
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

    pub fn extract_constraints(&self) -> (Vec<Vec<usize>>, Vec<SubSolutionSet>) {
        let cell_groups = self.cell_groups();
        let mut cell_groups_out = Vec::new();

        for (cell, group_id) in cell_groups
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.map(|a| (i, a)))
        {
            if group_id >= cell_groups_out.len() {
                cell_groups_out.resize(group_id + 1, Vec::new());
            }

            cell_groups_out[group_id].push(cell);
        }

        let full_mask: IntVec = cell_groups_out.iter().map(|l| l.len() as u8).collect();

        let subsolutions = self
            .grid
            .iter()
            .enumerate()
            .filter_map(|(i, hint)| {
                let Hint { remaining_mines, empties: 1.., .. } = hint else { return None };
                let mut mask = smallvec![0; cell_groups_out.len()];

                self.game.for_each_neighbor(i, |j| {
                    if self.grid[j] != Empty {
                        return;
                    }
                    let group = cell_groups[j].unwrap();
                    mask[group] = full_mask[group];
                });

                Some(SubSolutionSet::from_constraint(
                    mask,
                    *remaining_mines as usize,
                ))
            })
            .collect();

        (cell_groups_out, subsolutions)
    }

    pub fn remaining_mines(&self) -> usize {
        let placed_mines = self
            .grid
            .iter()
            .filter(|s| matches!(s, Mine { .. }))
            .count();

        self.game.num_mines() - placed_mines
    }

    pub fn remaining_empty_tiles(&self) -> usize {
        self.grid.iter().filter(|s| matches!(s, Empty)).count()
    }

    pub fn solve_csp(&mut self) -> Option<(Vec<Vec<usize>>, Vec<SubSolutionSet>)> {
        let mut tiles = Vec::new();

        loop {
            let (groups, mut subsolutions) = self.extract_constraints();

            if self.remaining_empty_tiles() == 0 {
                return None;
            }

            merge_all_subsolutions(&mut subsolutions);

            let (all_hints, all_mines) = solved_groups(&subsolutions);

            tiles.clear();

            for i in all_hints.iter_ones() {
                for tile in &groups[i] {
                    tiles.push(*tile);
                    self.uncover_tile(*tile)
                        .unwrap_or_else(|| panic!("Attempted to uncover mine at {tile:?}!"));
                }
            }

            for i in all_mines.iter_ones() {
                for tile in &groups[i] {
                    tiles.push(*tile);
                    self.flag_tile(*tile);
                }
            }

            if tiles.is_empty() {
                return Some((groups, subsolutions));
            }

            self.propogate(&mut tiles);
        }
    }

    pub fn get_solutionset(&self) -> Option<SolutionSet> {
        if self.remaining_empty_tiles() == 0 {
            return None;
        }

        let (groups, mut subsolutions) = self.extract_constraints();

        merge_all_subsolutions(&mut subsolutions);

        Some(SolutionSet::new(self, groups, subsolutions))
    }
}
