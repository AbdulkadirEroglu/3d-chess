//! @file search.rs
//! @brief AI search tests: finds mates and free material, promotes, stays deterministic.

use std::time::Instant;

use chess_ai::choose_move;
use chess_core::{Game, PieceKind};

/// @brief Loads a FEN position; panics on typos.
fn game(fen: &str) -> Game {
    Game::from_fen(fen).expect("valid fen")
}

/// @brief The chosen move as text like "a1a8".
fn chosen(game: &Game, depth: u32) -> String {
    let mv = choose_move(game, depth).expect("a move");
    format!("{}{}", mv.from, mv.to)
}

#[test]
fn finds_mate_in_one() {
    // Back-rank mate: the rook goes to a8.
    assert_eq!(
        chosen(&game("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1"), 3),
        "a1a8"
    );
}

#[test]
fn takes_a_free_queen() {
    assert_eq!(chosen(&game("k7/8/8/3q4/8/8/8/K2R4 w - - 0 1"), 3), "d1d5");
}

#[test]
fn promotes_to_a_queen() {
    let mv = choose_move(&game("8/P7/8/8/8/8/8/k6K w - - 0 1"), 3).expect("a move");
    assert_eq!(mv.promotion, Some(PieceKind::Queen));
}

#[test]
fn no_move_when_the_game_is_over() {
    // Stalemate: Black has no moves.
    assert_eq!(choose_move(&game("k7/8/1QK5/8/8/8/8/8 b - - 0 1"), 3), None);
}

#[test]
fn same_position_same_move() {
    let start = Game::new();
    let started = Instant::now();
    let first = chosen(&start, 4);
    // Printed with `cargo test -- --nocapture`; a rough speed check for the alpha depth.
    println!("depth 4 from the start position: {:?}", started.elapsed());
    assert_eq!(first, chosen(&start, 4));
}
