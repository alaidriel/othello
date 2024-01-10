pub use board::Piece;
use serde::{Deserialize, Serialize};
pub use state::State;

mod board;
mod state;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceError {
    #[error("board square ({0}, {1}) is occupied")]
    Occupied(usize, usize),
    #[error("it is not {0:?}'s turn")]
    Turn(Piece),
    #[error("board square ({0}, {1}) is not adjacent to any other piece")]
    NotAdjacent(usize, usize),
    #[error("board square ({0}, {1}) is out of bounds")]
    OutOfBounds(usize, usize),
    #[error("no pieces were flipped from board square ({0}, {1})")]
    NoFlips(usize, usize),
}
