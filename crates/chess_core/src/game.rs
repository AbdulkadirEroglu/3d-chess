//! @file game.rs
//! @brief The game: position, legal moves, applying move commands, and the result.

use cozy_chess as cc;
use tracing::{info, trace, warn};

use crate::command::{MoveCommand, MoveError};
use crate::types::{Color, Piece, PieceKind, Square};

/// @brief A legal move, described fully so the animation layer never has to work anything out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    /// @brief Square the piece leaves.
    pub from: Square,
    /// @brief Square the piece lands on. For castling, the king's square (e.g. g1).
    pub to: Square,
    /// @brief The piece that moves (the king, when castling).
    pub piece: Piece,
    /// @brief What a pawn becomes, if it promotes.
    pub promotion: Option<PieceKind>,
    /// @brief The captured piece and the square it stood on. Differs from `to` for en passant.
    pub capture: Option<(Piece, Square)>,
    /// @brief For castling: the rook's from and to squares.
    pub rook_move: Option<(Square, Square)>,
}

/// @brief Whether the game is still going, or how it ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    /// @brief The game continues.
    Ongoing,
    /// @brief One side is checkmated.
    Checkmate {
        /// @brief The side that delivered mate.
        winner: Color,
    },
    /// @brief The side to move has no legal moves and is not in check.
    Stalemate,
    /// @brief 50 moves by each side without a capture or pawn move.
    FiftyMoveRule,
    /// @brief The same position occurred three times.
    ThreefoldRepetition,
    /// @brief Neither side has enough pieces left to checkmate.
    InsufficientMaterial,
}

/// @brief A chess game: the current position, its repetition history and the result.
/// @note Draws by the 50-move rule and threefold repetition end the game automatically (no claim needed).
#[derive(Clone, Debug)]
pub struct Game {
    /// @brief Current position. Rules-library type, never exposed.
    board: cc::Board,
    /// @brief Positions since the last capture or pawn move, for threefold repetition.
    history: Vec<cc::Board>,
    /// @brief Result, updated after every move.
    result: GameResult,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// @brief A new game from the standard starting position.
    pub fn new() -> Self {
        Self::from_board(cc::Board::default())
    }

    /// @brief A game from a FEN position, e.g. for tests or puzzles.
    /// @return None if the FEN text is invalid.
    pub fn from_fen(fen: &str) -> Option<Self> {
        fen.parse().ok().map(Self::from_board)
    }

    /// @brief Wraps a library board into a game and works out its result.
    fn from_board(board: cc::Board) -> Self {
        let mut game = Self {
            history: vec![board.clone()],
            board,
            result: GameResult::Ongoing,
        };
        game.result = game.compute_result();
        game
    }

    /// @brief The side whose turn it is.
    pub fn side_to_move(&self) -> Color {
        color_from(self.board.side_to_move())
    }

    /// @brief Whether the game is still going, or how it ended.
    pub fn result(&self) -> GameResult {
        self.result
    }

    /// @brief The piece on a square, if any.
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let square = cc::Square::index(usize::from(square.index()));
        Some(Piece {
            kind: kind_from(self.board.piece_on(square)?),
            color: color_from(self.board.color_on(square)?),
        })
    }

    /// @brief Every piece on the board with its square.
    pub fn pieces(&self) -> impl Iterator<Item = (Square, Piece)> + '_ {
        (0..64).filter_map(|index| {
            let square = Square(index);
            Some((square, self.piece_at(square)?))
        })
    }

    /// @brief All legal moves for the side to move. Empty once the game is over.
    pub fn legal_moves(&self) -> Vec<Move> {
        self.legal_pairs().into_iter().map(|(_, mv)| mv).collect()
    }

    /// @brief Checks a move command and plays it if it's legal.
    /// @param cmd Move requested by a player, the AI or the network.
    /// @return The move as played, or why it was rejected.
    pub fn apply(&mut self, cmd: MoveCommand) -> Result<Move, MoveError> {
        let candidates: Vec<_> = self
            .legal_pairs()
            .into_iter()
            .filter(|(_, mv)| mv.from == cmd.from && mv.to == cmd.to)
            .collect();

        let Some(&(raw, mv)) = candidates
            .iter()
            .find(|(_, mv)| mv.promotion == cmd.promotion)
        else {
            let error = match self.piece_at(cmd.from) {
                _ if self.result != GameResult::Ongoing => MoveError::GameOver,
                None => MoveError::NoPiece(cmd.from),
                Some(piece) if piece.color != self.side_to_move() => {
                    MoveError::NotYourPiece(cmd.from)
                }
                // The squares match a legal move, only the promotion piece is missing.
                Some(_) if cmd.promotion.is_none() && !candidates.is_empty() => {
                    MoveError::PromotionRequired
                }
                Some(_) => MoveError::Illegal,
            };
            warn!(?cmd, %error, "move rejected");
            return Err(error);
        };

        self.board.play_unchecked(raw);
        // A capture or pawn move makes earlier positions unreachable, so they can't repeat.
        if self.board.halfmove_clock() == 0 {
            self.history.clear();
        }
        self.history.push(self.board.clone());
        self.result = self.compute_result();

        // trace, not info: the AI and perft apply moves in bulk. The game crate logs played moves.
        trace!(?cmd, ?mv, "move applied");
        if self.result != GameResult::Ongoing {
            info!(result = ?self.result, "game over");
        }
        Ok(mv)
    }

    /// @brief Legal moves as (library move, our move) pairs, so `apply` can play the library move.
    fn legal_pairs(&self) -> Vec<(cc::Move, Move)> {
        let mut pairs = Vec::new();
        if self.result != GameResult::Ongoing {
            return pairs;
        }
        self.board.generate_moves(|piece_moves| {
            pairs.extend(
                piece_moves
                    .into_iter()
                    .filter_map(|raw| Some((raw, self.describe(raw)?))),
            );
            false
        });
        pairs
    }

    /// @brief Turns a library move into our `Move`: standard castling squares, capture and rook details.
    /// @return None only if the move doesn't belong to this position.
    fn describe(&self, raw: cc::Move) -> Option<Move> {
        let from = square_from(raw.from);
        let target = square_from(raw.to);
        let piece = self.piece_at(from)?;

        // The library writes castling as "king takes own rook" (e1h1); players expect e1g1.
        let own_piece_on_target = self
            .piece_at(target)
            .is_some_and(|other| other.color == piece.color);
        if piece.kind == PieceKind::King && own_piece_on_target {
            let short = target.file() > from.file();
            let (king_file, rook_file) = if short { (6, 5) } else { (2, 3) };
            return Some(Move {
                from,
                to: Square::new(king_file, from.rank())?,
                piece,
                promotion: None,
                capture: None,
                rook_move: Some((target, Square::new(rook_file, from.rank())?)),
            });
        }

        let capture = match self.piece_at(target) {
            Some(captured) => Some((captured, target)),
            // En passant: a pawn moving diagonally onto an empty square takes the pawn beside it.
            None if piece.kind == PieceKind::Pawn && target.file() != from.file() => {
                let square = Square::new(target.file(), from.rank())?;
                Some((self.piece_at(square)?, square))
            }
            None => None,
        };

        Some(Move {
            from,
            to: target,
            piece,
            promotion: raw.promotion.map(kind_from),
            capture,
            rook_move: None,
        })
    }

    /// @brief Works out whether the game has ended, and how.
    /// @note Checkmate and stalemate are checked first: they take precedence over the draw rules.
    fn compute_result(&self) -> GameResult {
        let board = &self.board;
        let has_moves = board.generate_moves(|_| true);
        if !has_moves {
            return if board.checkers().is_empty() {
                GameResult::Stalemate
            } else {
                GameResult::Checkmate {
                    winner: self.side_to_move().opponent(),
                }
            };
        }
        if board.halfmove_clock() >= 100 {
            return GameResult::FiftyMoveRule;
        }
        let repetitions = self
            .history
            .iter()
            .filter(|earlier| earlier.same_position(board))
            .count();
        if repetitions >= 3 {
            return GameResult::ThreefoldRepetition;
        }

        // Insufficient material: no pawns, rooks or queens, and at most one minor piece,
        // or only bishops that all stand on the same square color.
        let heavy = board.pieces(cc::Piece::Pawn)
            | board.pieces(cc::Piece::Rook)
            | board.pieces(cc::Piece::Queen);
        let knights = board.pieces(cc::Piece::Knight);
        let bishops = board.pieces(cc::Piece::Bishop);
        let same_color_bishops = knights.is_empty()
            && (bishops.is_subset(cc::BitBoard::DARK_SQUARES)
                || bishops.is_subset(cc::BitBoard::LIGHT_SQUARES));
        if heavy.is_empty() && ((knights | bishops).len() <= 1 || same_color_bishops) {
            return GameResult::InsufficientMaterial;
        }

        GameResult::Ongoing
    }
}

/// @brief Library square to our square. Both use a1 = 0 … h8 = 63.
fn square_from(square: cc::Square) -> Square {
    Square(square as u8)
}

/// @brief Library color to our color.
fn color_from(color: cc::Color) -> Color {
    match color {
        cc::Color::White => Color::White,
        cc::Color::Black => Color::Black,
    }
}

/// @brief Library piece to our piece kind.
fn kind_from(piece: cc::Piece) -> PieceKind {
    match piece {
        cc::Piece::Pawn => PieceKind::Pawn,
        cc::Piece::Knight => PieceKind::Knight,
        cc::Piece::Bishop => PieceKind::Bishop,
        cc::Piece::Rook => PieceKind::Rook,
        cc::Piece::Queen => PieceKind::Queen,
        cc::Piece::King => PieceKind::King,
    }
}
