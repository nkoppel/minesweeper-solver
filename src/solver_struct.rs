use itertools::Itertools;

use crate::board::*;
use crate::game::*;

#[derive(Debug, PartialEq)]
pub struct Solver<Gr: Graph, Ga: Game<Graph = Gr>> {
    board: Board<Gr>,
    game: Ga,
}

impl<Gr: Graph, Ga: Game<Graph = Gr>> Game for Solver<Gr, Ga> {
    type Graph = Gr;

    fn graph(&self) -> &Self::Graph {
        self.game.graph()
    }

    fn explore_tile(&mut self, pos: usize) -> Option<u8> {
        self.game.explore_tile(pos)
    }

    fn num_mines(&self) -> usize {
        self.game.num_mines()
    }
}

impl<Gr: Graph, Ga: Game<Graph = Gr>> Solver<Gr, Ga> {
    pub fn new(board: Board<Gr>, game: Ga) -> Self {
        assert!(board.graph == *game.graph());
        Self { board, game }
    }

    pub fn from_game(game: Ga) -> Self {
        let board = Board::from_game(&game);
        Self { board, game }
    }

    pub fn board(&self) -> &Board<Gr> {
        &self.board
    }

    pub fn game(&self) -> &Ga {
        &self.game
    }

    pub fn into_board(self) -> Board<Gr> {
        self.board
    }

    pub fn into_game(self) -> Ga {
        self.game
    }

    pub fn decompose(self) -> (Board<Gr>, Ga) {
        (self.board, self.game)
    }

    pub fn clear_tile(&mut self, tile: usize) {
        self.board.clear_tile(tile)
    }

    pub fn flag_tile(&mut self, tile: usize) {
        self.board.flag_tile(tile)
    }

    pub fn assert_tile(&mut self, tile: usize) {
        self.board.assert_tile(tile)
    }

    pub fn set_tile(&mut self, tile: usize, hint: u8) {
        self.board.set_tile(tile, hint)
    }

    pub fn remaining_mines(&self) -> usize {
        self.board.remaining_mines()
    }

    pub fn remaining_empty_tiles(&self) -> usize {
        self.board.remaining_empty_tiles()
    }

    pub fn is_solved(&self) -> bool {
        self.board.is_solved()
    }

    #[must_use]
    pub fn uncover_tile(&mut self, tile: usize) -> Option<()> {
        if self.board.grid[tile] != Empty {
            return Some(());
        }

        let hint = self.game.explore_tile(tile)?;
        self.board.set_tile(tile, hint);

        Some(())
    }

    #[must_use]
    fn propogate_tile(&mut self, loc: usize, neighbors: &[usize]) -> Option<bool> {
        let tile = &mut self.board.grid[loc];

        match tile {
            Mine {
                needs_propogate: ref mut needs_propogate @ true,
            }
            | AssertHint {
                needs_propogate: ref mut needs_propogate @ true,
            } => {
                *needs_propogate = false;
                Some(true)
            }

            Hint { .. } if tile.needs_hint_fill() => {
                for n in neighbors {
                    self.uncover_tile(*n)?;
                }
                Some(true)
            }
            Hint { .. } if tile.needs_flag_fill() => {
                for n in neighbors {
                    self.board.flag_tile(*n);
                }
                Some(true)
            }

            _ => Some(false),
        }
    }
}

impl<Ga: Game<Graph = Graph2d>> Solver<Graph2d, Ga> {
    #[must_use]
    pub fn propogate(&mut self, tiles: &mut Vec<usize>) -> Option<()> {
        let stack = tiles;
        let mut neighbors = Vec::with_capacity(8);

        while let Some(loc) = stack.last().copied() {
            neighbors.clear();
            neighbors.extend(self.board.neighbors(loc));

            self.propogate_tile(loc, &neighbors)?;

            if let Some(next) = neighbors
                .iter()
                .find(|n| self.board.grid[**n].needs_propogate())
            {
                stack.push(*next);
            } else {
                stack.pop();
            }
        }

        Some(())
    }

    #[must_use]
    pub fn solve(&mut self) -> Option<()> {
        let mut tiles = self
            .board
            .grid
            .iter()
            .positions(|tile| tile.needs_propogate())
            .collect();

        loop {
            self.propogate(&mut tiles)?;

            if self.is_solved() {
                break;
            }

            let (safe, mines) = self.board.solutionset().solved();

            if !safe.any() {
                break;
            }

            for tile in safe.iter_ones() {
                self.uncover_tile(tile)?;
                tiles.push(tile);
            }

            for tile in mines.iter_ones() {
                self.flag_tile(tile);
                tiles.push(tile);
            }
        }

        Some(())
    }
}
