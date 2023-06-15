use crate::game::*;
use crate::solver::{csp::*, *};

use itertools::Itertools;
use smallvec::*;

use super::solutionset::SolutionSet;

impl<G: Game> Solver<G> {
    /// Assigns a group id to each empty cell so that empty cells with the same id are constrained
    /// by the same set of hints.
    fn cell_groups(&self) -> Vec<Option<usize>> {
        let mut table: Vec<usize> = Vec::with_capacity(self.grid.len());
        let mut flags: Vec<bool> = Vec::with_capacity(self.grid.len());
        let mut ids: Vec<usize> = vec![0; self.grid.len()];

        let mut max_group = 1;

        table.push(0);
        flags.push(false);

        for i in self
            .grid
            .iter()
            .positions(|c| matches!(c, Hint { empties: 1.., .. }))
        {
            self.game.for_each_neighbor(i, |j| {
                if self.grid[j] != Empty {
                    return;
                }

                let id = ids[j];

                if !flags[id] {
                    table[id] = max_group;
                    flags[id] = true;
                    max_group += 1;

                    table.push(0);
                    flags.push(false);
                }
            });

            self.game.for_each_neighbor(i, |j| {
                if self.grid[j] != Empty {
                    return;
                }

                let id = ids[j];

                ids[j] = table[id];
                flags[id] = false;
            });
        }

        max_group = 0;

        let mut out = vec![None; self.grid.len()];

        for (i, id) in ids.iter_mut().enumerate() {
            if self.grid[i] != Empty || *id == 0 {
                continue;
            }

            if !flags[*id] {
                table[*id] = max_group;
                flags[*id] = true;
                max_group += 1;
            }

            out[i] = Some(table[*id]);
        }

        out
    }

    pub fn extract_constraints(&self) -> (Vec<Vec<usize>>, Vec<SubSolutionSet>) {
        let cell_groups = self.cell_groups();
        let mut cell_groups_out = Vec::new();

        for (cell, group_id) in cell_groups
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.map(|y| (i, y)))
        {
            if group_id >= cell_groups_out.len() {
                cell_groups_out.resize(group_id + 1, Vec::new());
            }

            cell_groups_out[group_id].push(cell);
        }

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
                    mask[group] = cell_groups_out[group].len() as u8;
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

    pub fn remaining_empty_squares(&self) -> usize {
        self.grid.iter().filter(|s| matches!(s, Empty)).count()
    }

    pub fn solve_csp(&mut self) -> Option<SolutionSet<G>> {
        let mut squares = Vec::new();

        loop {
            let (groups, mut subsolutions) = self.extract_constraints();

            if subsolutions.is_empty() {
                return None;
            }

            merge_all_subsolutions(&mut subsolutions);

            let (all_hints, all_mines) = solved_groups(&subsolutions);

            squares.clear();

            for i in all_hints.iter_ones() {
                for square in &groups[i] {
                    squares.push(*square);
                    self.uncover_square(*square)
                        .unwrap_or_else(|| panic!("Attempted to uncover mine at {square:?}!"));
                }
            }

            for i in all_mines.iter_ones() {
                for square in &groups[i] {
                    squares.push(*square);
                    self.flag_square(*square);
                }
            }

            if squares.is_empty() {
                return Some(SolutionSet::new(self, groups, subsolutions));
            }

            self.propogate(&mut squares);
        }
    }
}
