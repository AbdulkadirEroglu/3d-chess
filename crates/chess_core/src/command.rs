//! @file command.rs
//! @brief Move requests from any input source, and why they can be rejected.

use crate::types::{PieceKind, Square};

/// @brief Where a move request came from. Used in logs and to send feedback to the right place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandSource {
    /// @brief Mouse or touch input.
    Pointer,
    /// @brief The computer opponent.
    Ai,
}

/// @brief A request to make a move. Every input source produces one; `Game::apply` decides if it's legal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveCommand {
    /// @brief Square of the piece to move.
    pub from: Square,
    /// @brief Target square. For castling, the king's target (e.g. g1), as players see it.
    pub to: Square,
    /// @brief Piece to promote to; required when a pawn reaches the last rank.
    pub promotion: Option<PieceKind>,
    /// @brief Who sent the request.
    pub source: CommandSource,
}

/// @brief Why a `MoveCommand` was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MoveError {
    /// @brief The game has already ended.
    #[error("the game is already over")]
    GameOver,
    /// @brief There is no piece on the `from` square.
    #[error("no piece on {0}")]
    NoPiece(Square),
    /// @brief The piece on the `from` square belongs to the side not on move.
    #[error("the piece on {0} belongs to the other side")]
    NotYourPiece(Square),
    /// @brief A pawn reaches the last rank but no promotion piece was given.
    #[error("a pawn reaching the last rank needs a promotion piece")]
    PromotionRequired,
    /// @brief The move breaks the rules.
    #[error("illegal move")]
    Illegal,
}
