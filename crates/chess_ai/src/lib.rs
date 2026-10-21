//! @file lib.rs
//! @brief Computer opponent: alpha-beta search over `chess_core` positions. No Bevy.
//! @note Deterministic: the same position and depth always give the same move.

use std::cmp::Reverse;
use std::time::Instant;

use chess_core::{Color, CommandSource, Game, GameResult, Move, MoveCommand, PieceKind};
use tracing::debug;

/// @brief Score for checkmate. Mates found in fewer moves score slightly higher.
const MATE_SCORE: i32 = 1_000_000;

/// @brief Bound wider than any score; safe to negate.
const INFINITY: i32 = 2 * MATE_SCORE;

/// @brief Picks the best move for the side to move, looking `depth` plies (half-moves) ahead.
/// @return None when the game is already over.
pub fn choose_move(game: &Game, depth: u32) -> Option<Move> {
    let started = Instant::now();
    let mut nodes = 0;
    let mut best: Option<(Move, i32)> = None;
    let mut alpha = -INFINITY;
    for mv in ordered_moves(game) {
        let Some(next) = play(game, mv) else {
            continue;
        };
        let score = -negamax(&next, depth.max(1) - 1, -INFINITY, -alpha, 1, &mut nodes);
        // Strictly better only, so ties keep the earlier move and the choice stays deterministic.
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((mv, score));
            alpha = alpha.max(score);
        }
    }
    debug!(
        depth,
        nodes,
        ms = started.elapsed().as_millis(),
        best = ?best.map(|(mv, score)| (mv.from, mv.to, score)),
        "search done"
    );
    best.map(|(mv, _)| mv)
}

/// @brief Negamax with alpha-beta pruning.
/// @return Score from the point of view of the side to move in `game`.
fn negamax(game: &Game, depth: u32, mut alpha: i32, beta: i32, ply: i32, nodes: &mut u64) -> i32 {
    *nodes += 1;
    match game.result() {
        GameResult::Ongoing => {}
        // The side to move is mated; the further away, the less bad.
        GameResult::Checkmate { .. } => return -MATE_SCORE + ply,
        _ => return 0,
    }
    if depth == 0 {
        return evaluate(game);
    }
    for mv in ordered_moves(game) {
        let Some(next) = play(game, mv) else {
            continue;
        };
        let score = -negamax(&next, depth - 1, -beta, -alpha, ply + 1, nodes);
        if score >= beta {
            return beta;
        }
        alpha = alpha.max(score);
    }
    alpha
}

/// @brief The game after a legal move.
fn play(game: &Game, mv: Move) -> Option<Game> {
    let mut next = game.clone();
    next.apply(MoveCommand {
        from: mv.from,
        to: mv.to,
        promotion: mv.promotion,
        source: CommandSource::Ai,
    })
    .ok()?;
    Some(next)
}

/// @brief Legal moves with the most promising first, so alpha-beta prunes more.
/// @note Captures of valuable pieces by cheap ones first, then promotions, then the rest.
fn ordered_moves(game: &Game) -> Vec<Move> {
    let mut moves = game.legal_moves();
    moves.sort_by_key(|mv| {
        let capture = mv.capture.map_or(0, |(victim, _)| {
            10 * value(victim.kind) - value(mv.piece.kind)
        });
        let promotion = mv.promotion.map_or(0, value);
        Reverse(capture + promotion)
    });
    moves
}

/// @brief Material value of a piece kind in centipawns.
fn value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0,
    }
}

/// @brief Static score of a position for the side to move: material plus simple placement bonuses.
/// @note Knights and bishops like the center; pawns like to advance.
fn evaluate(game: &Game) -> i32 {
    let us = game.side_to_move();
    game.pieces()
        .map(|(square, piece)| {
            // 0 on the edge, 6 on the four center squares.
            let center = i32::from(square.file().min(7 - square.file()))
                + i32::from(square.rank().min(7 - square.rank()));
            let advanced = i32::from(match piece.color {
                Color::White => square.rank(),
                Color::Black => 7 - square.rank(),
            });
            let bonus = match piece.kind {
                PieceKind::Pawn => 8 * advanced,
                PieceKind::Knight => 8 * center,
                PieceKind::Bishop => 4 * center,
                PieceKind::Queen => 2 * center,
                PieceKind::Rook | PieceKind::King => 0,
            };
            let score = value(piece.kind) + bonus;
            if piece.color == us { score } else { -score }
        })
        .sum()
}
