use crate::game::*;
use crate::solver::*;

impl Solver {
    fn new_field(&mut self, point: Point) -> usize {
        self.field_id += 1;
        let id = self.field_id;

        for point2 in self.game.get_neighbors(point) {
            let square = self.get_point_mut(point2);

            if *square == Empty {
                *square = Active(HashSet::with_capacity(8));
            }

            if let Active(set) = square {
                set.insert(id);
            }
        }

        id
    }

    fn solve_field(&mut self, f: usize) -> bool {
        if let Some(field) = self.fields.get(&f) {
            match field.solved_status() {
                Some(true) => {
                    let field = self.fields.remove(&f).unwrap();

                    for point in field.points {
                        self.set_point(point, Mine);
                    }

                    return true;
                }
                Some(false) => {
                    let field = self.fields.remove(&f).unwrap();

                    for point in field.points {
                        self.set_point(point, Mine);
                    }

                    return true;
                }
                None => return false,
            }
        }
        false
    }

    fn field_remove_point(&mut self, field: usize, point: Point) -> bool {
        let is_mine = *self.get_point(point) == Mine;

        if let Some(f) = self.fields.get_mut(&field) {
            let out = f.points.remove(&point);

            if is_mine {
                f.nmines -= 1;
            }

            if f.points.is_empty() {
                self.fields.remove(&field);
            }

            if let Active(set) = self.get_point_mut(point) {
                set.remove(&field);
            }

            return out;
        }
        false
    }

    fn field_remove_field(&mut self, f1: usize, f2: usize) {
        let field1 = self.fields.get(&f1).unwrap();
        let field2 = self.fields.get(&f2).unwrap();

        let intersect: HashSet<_> =
            field1.points
                .intersection(&field2.points)
                .map(|x| *x)
                .collect();

        let points: HashSet<_> =
            field1.points
                .difference(&intersect)
                .map(|x| *x)
                .collect();

        let mines2 = field2.nmines;

        let field1 = self.fields.get_mut(&f1).unwrap();
        field1.nmines -= mines2;
        field1.points = points;

        if field1.points.is_empty() {
            self.fields.remove(&f1);
        }

        for point in intersect {
            if let Active(set) = self.get_point_mut(point) {
                set.remove(&f1);
            }
        }
    }

    fn field_get_collisions(&self, f: usize) -> HashSet<usize> {
        let mut out = HashSet::new();

        for p in &self.fields.get(&f).unwrap().points {
            if let Active(set) = self.get_point(*p) {
                for f2 in set {
                    out.insert(*f2);
                }
            }
        }

        out.remove(&f);

        out
    }

    pub fn propogate(&mut self, point: Point) {
        let mut stack = self.game.get_neighbors(point);

        stack.push(point);

        while let Some(point) = stack.pop() {
            if let Num(nmines) = self.get_point(point) {
                let mut neighbors = self.game.get_neighbors(point);
                let mut nmines_found = 0;
                let mut nempty = 0;

                for point2 in &neighbors {
                    match self.get_point(*point2) {
                        Mine => nmines_found += 1,
                        Empty | Active(_) => nempty += 1,
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
                        match self.get_point(*point2) {
                            Empty | Active(_) => {
                                self.set_point(*point2, Mine);
                                changed = true;
                            },
                            _ => ()
                        }
                    }

                    if changed {
                        stack.append(&mut neighbors);
                        stack.append(&mut self.game.get_double_neighbors(point));
                    }
                }
            }
        }
    }

    fn propogate_fields(&mut self, f: usize) {
        if self.solve_field(f) {
            return;
        }

        let field1 = self.fields.get(&f).unwrap();

        for f2 in self.field_get_collisions(f) {
            let field2 = self.fields.get(&f2).unwrap();

            let len1 = field1.points.len();
            let len2 = field2.points.len();
            let ilen = field1.points.intersection(&field2.points).count();

            if ilen == len1 {
                
            }
        }
    }
}
