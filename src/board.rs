use std::{
    fmt,
    ops::{Index, IndexMut, Not},
};

use serde::{Deserialize, Serialize};

use crate::PlaceError;

const DIRECTIONS: &[(i8, i8)] = &[
    (-1, -1), // Left diagonal
    (0, -1),  // Top
    (1, -1),  // Top right
    (-1, 0),  // Left
    (1, 0),   // Right
    (-1, 1),  // Bottom left
    (0, 1),   // Bottom
    (1, 1),   // Bottom right
];
const WIDTH: i8 = Board::width() as i8;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Piece {
    Black,
    White,
}

impl Not for Piece {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Board(Vec<Option<Piece>>);

impl Board {
    /// Initializes an Othello board with the standard starting state.
    pub fn new() -> Self {
        /*
        ........
        ........
        ........
        ...●○...
        ...○●...
        ........
        ........
        ........
        */
        let mut board = Self(vec![None; Self::width() * Self::width()]);
        board[(3, 3)] = Some(Piece::White); // Top left
        board[(4, 3)] = Some(Piece::Black); // Top right
        board[(3, 4)] = Some(Piece::Black); // Bottom left
        board[(4, 4)] = Some(Piece::White); // Bottom right
        board
    }

    pub fn adjacent(&self, x: usize, y: usize) -> Result<bool, PlaceError> {
        // Calling code ensures that x and y are within bounds.
        assert!(x < Self::width() && y < Self::width());
        let x = i8::try_from(x).unwrap();
        let y = i8::try_from(y).unwrap();
        Ok(DIRECTIONS
            .iter()
            .filter(|(dx, dy)| x + dx >= 0 && y + dy >= 0)
            .map(|(dx, dy)| (x + dx, y + dy))
            .map(|(x, y)| (usize::try_from(x).unwrap(), usize::try_from(y).unwrap()))
            .filter(|&(x, y)| x < Self::width() && y < Self::width())
            .any(|(x, y)| self[(x, y)].is_some()))
    }

    pub fn flips(&self, x: usize, y: usize, piece: Piece) -> usize {
        // Calling code ensures that x and y are within bounds.
        assert!(x < Self::width() && y < Self::width());
        let mut count = 0;
        let x = i8::try_from(x).unwrap();
        let y = i8::try_from(y).unwrap();
        for (dx, dy) in DIRECTIONS {
            if self.on((x, y), (dx, dy), piece) {
                self.walk((x, y), (dx, dy), piece, |_, _| count += 1);
            }
        }
        count
    }

    pub fn flip(&mut self, x: usize, y: usize, piece: Piece) {
        // Calling code ensures that x and y are within bounds.
        assert!(x < Self::width() && y < Self::width());
        let x = i8::try_from(x).unwrap();
        let y = i8::try_from(y).unwrap();
        for (dx, dy) in DIRECTIONS {
            if self.on((x, y), (dx, dy), piece) {
                self.walk_mut((x, y), (dx, dy), piece, |board, cur| {
                    board[cur] = Some(piece);
                });
            }
        }
    }

    // TODO: fix duplication
    fn walk<R, F>(&self, (x, y): (i8, i8), (dx, dy): (&i8, &i8), piece: Piece, mut f: F)
    where
        F: FnMut(&Self, (usize, usize)) -> R,
    {
        let opponent = !piece;
        let mut x = x + dx;
        let mut y = y + dy;
        while x >= 0 && y >= 0 && x < WIDTH && y < WIDTH {
            let cur = (usize::try_from(x).unwrap(), usize::try_from(y).unwrap());
            match self[cur] {
                Some(p) if p == opponent => {
                    f(self, cur);
                    x += dx;
                    y += dy;
                }
                Some(p) if p == piece => break,
                _ => break,
            }
        }
    }

    // TODO: see walk()
    fn walk_mut<R, F>(&mut self, (x, y): (i8, i8), (dx, dy): (&i8, &i8), piece: Piece, mut f: F)
    where
        F: FnMut(&mut Self, (usize, usize)) -> R,
    {
        let opponent = !piece;
        let mut x = x + dx;
        let mut y = y + dy;
        while x >= 0 && y >= 0 && x < WIDTH && y < WIDTH {
            let cur = (usize::try_from(x).unwrap(), usize::try_from(y).unwrap());
            match self[cur] {
                Some(p) if p == opponent => {
                    f(self, cur);
                    x += dx;
                    y += dy;
                }
                Some(p) if p == piece => break,
                _ => break,
            }
        }
    }

    fn on(&self, (x, y): (i8, i8), (dx, dy): (&i8, &i8), piece: Piece) -> bool {
        let mut x = x + dx;
        let mut y = y + dy;
        while x >= 0 && y >= 0 && x < WIDTH && y < WIDTH {
            let cur = (usize::try_from(x).unwrap(), usize::try_from(y).unwrap());
            if self[cur].is_some_and(|square| piece == square) {
                return true;
            }
            x += dx;
            y += dy;
        }
        false
    }

    /// The width of the board. A standard Othello board is an 8x8 grid.
    pub const fn width() -> usize {
        8
    }
}

impl Index<(usize, usize)> for Board {
    type Output = Option<Piece>;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        let i = x + y * Self::width();
        &self.0[i]
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let i = x + y * Self::width();
        &mut self.0[i]
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, piece) in self.0.iter().enumerate() {
            let c = match piece {
                Some(Piece::Black) => '○',
                Some(Piece::White) => '●',
                None => '.',
            };
            write!(f, "{}", c)?;
            if i % 8 == 7 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent() {
        let board = Board::new();
        assert!(!board.adjacent(0, 0).unwrap());
        assert!(board.adjacent(2, 3).unwrap());
    }
}
