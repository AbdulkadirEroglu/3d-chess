//! @file types.rs
//! @brief Board vocabulary: squares, colors and pieces. Our own types, so the rules library never leaks out.

use std::fmt;
use std::str::FromStr;

/// @brief One of the 64 board locations, a1 to h8.
/// @note Stored as an index: a1 = 0, b1 = 1, …, h1 = 7, a2 = 8, …, h8 = 63.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square(pub(crate) u8);

impl Square {
    /// @brief Builds a square from its file (0 = a … 7 = h) and rank (0 = rank 1 … 7 = rank 8).
    /// @return None if either value is 8 or more.
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        (file < 8 && rank < 8).then_some(Self(rank * 8 + file))
    }

    /// @brief Builds a square from its index (a1 = 0 … h8 = 63).
    /// @return None if the index is 64 or more.
    pub fn from_index(index: u8) -> Option<Self> {
        (index < 64).then_some(Self(index))
    }

    /// @brief Index of the square, a1 = 0 … h8 = 63.
    pub fn index(self) -> u8 {
        self.0
    }

    /// @brief Board column: 0 = a … 7 = h.
    pub fn file(self) -> u8 {
        self.0 % 8
    }

    /// @brief Board row: 0 = rank 1 … 7 = rank 8.
    pub fn rank(self) -> u8 {
        self.0 / 8
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", char::from(b'a' + self.file()), self.rank() + 1)
    }
}

// Debug prints "e4" too, so logs stay readable.
impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// @brief Error for text that isn't a square name like "e4".
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a square: {0:?}")]
pub struct ParseSquareError(pub String);

impl FromStr for Square {
    type Err = ParseSquareError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let square = match s.as_bytes() {
            [file @ b'a'..=b'h', rank @ b'1'..=b'8'] => Square::new(file - b'a', rank - b'1'),
            _ => None,
        };
        square.ok_or_else(|| ParseSquareError(s.to_string()))
    }
}

/// @brief One of the two sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// @brief Moves first.
    White,
    /// @brief Moves second.
    Black,
}

impl Color {
    /// @brief The other side.
    pub fn opponent(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// @brief What kind of piece something is, regardless of color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceKind {
    /// @brief Pawn.
    Pawn,
    /// @brief Knight.
    Knight,
    /// @brief Bishop.
    Bishop,
    /// @brief Rook.
    Rook,
    /// @brief Queen.
    Queen,
    /// @brief King.
    King,
}

/// @brief A piece on the board: its kind and its color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    /// @brief Pawn, knight, …
    pub kind: PieceKind,
    /// @brief The side it belongs to.
    pub color: Color,
}
