use crate::game::*;
use crate::solver::*;
use crate::csp::*;
use crate::bitvec::*;

use std::collections::HashMap;

impl Solver {
    pub fn propogate(&mut self, point: Point) {
        let mut stack = self.game.get_neighbors(point);

        stack.push(point);

        while let Some(point) = stack.pop() {
            let Num(nmines) = self.get_point(point) else {continue};

            let neighbors = self.game.get_neighbors(point);
            let mut nmines_found = 0;
            let mut nempty = 0;

            for point2 in &neighbors {
                match self.get_point(*point2) {
                    Mine => nmines_found += 1,
                    Empty => nempty += 1,
                    _ => ()
                }
            }

            if nempty == 0 {
                continue;
            } else if nmines_found == *nmines {
                let mut changed = false;

                for point2 in &neighbors {
                    if *self.get_point(*point2) != Mine {
                        self.uncover_point(*point2);
                        changed = true;
                    }
                }

                if changed {
                    stack.append(&mut self.game.get_neighbors(point));
                }
            } else if nempty == nmines - nmines_found {
                let mut changed = false;

                for point2 in &neighbors {
                    if *self.get_point(*point2) == Empty {
                        self.set_point(*point2, Mine);
                        changed = true;
                    }
                }

                if changed {
                    stack.append(&mut self.game.get_double_neighbors(point));
                }
            }
        }
    }

    pub fn extract_constraints(&self) -> (Vec<Point>, Vec<(BitVec, usize)>) {
        let mut points1 = HashMap::new();
        let mut points2 = Vec::new();
        let mut rows = Vec::new();

        let (width, height) = self.game.size();

        for y in 0..height {
            for x in 0..width {
                let point = (x, y);

                let Num(mut nmines) = self.get_point(point) else {continue};

                let mut row = (BitVec::new(false, points2.len()), 0);
                let mut has_empty = false;

                for point2 in self.game.get_neighbors(point) {
                    if *self.get_point(point2) != Empty {
                        if *self.get_point(point2) == Mine {
                            nmines -= 1;
                        }
                        continue;
                    }
                    has_empty = true;

                    if let Some(id) = points1.get(&point2) {
                        row.0.set(*id, true);
                    } else {
                        points1.insert(point2, points2.len());
                        points2.push(point2);
                        row.0.push(true);
                    }
                }

                row.1 = nmines;

                if has_empty {
                    rows.push(row);
                }
            }
        }

        for row in &mut rows {
            row.0.resize(points2.len(), false);
        }

        // println!("{points1:?}");

        (points2, rows)
    }

    pub fn solve_csp(&mut self) {
        let (points, rows) = self.extract_constraints();
        let mut subsolutions = rows
            .into_iter()
            .map(|(mask, count)| SubSolutionSet::from_constraint(mask, count))
            .collect::<Vec<_>>();

        // println!("{subsolutions:?}");

        merge_all_subsolutions(&mut subsolutions);

        // println!("{points:?}");
        // println!("{}", points.len());
        // println!();
        // println!("{subsolutions:?}");
    }
}
