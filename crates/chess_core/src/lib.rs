//! @file lib.rs
//! @brief Chess rules and game state. Plain Rust, no Bevy.
//! @note `cozy-chess` is used internally and never appears in the public API; it gets replaced
//!       by our own move generator before v1.0.

mod command;
mod game;
mod types;

pub use command::{CommandSource, MoveCommand, MoveError};
pub use game::{Game, GameResult, Move};
pub use types::{Color, ParseSquareError, Piece, PieceKind, Square};
