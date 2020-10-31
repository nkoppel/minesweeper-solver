use std::cell::Cell;
use std::rc::Rc;

use std::collections::HashSet;

mod game;

struct Field {
    pub min_mines: usize,
    pub max_mines: usize,
    pub squares: HashSet<(usize, usize)>,
}

type FieldRef = Rc<Cell<Field>>;

enum Square {
    Empty,
    Mine,
    Num(usize),
    Active(Vec<FieldRef>)
}

fn main() {
    println!("Hello, world!");
}
