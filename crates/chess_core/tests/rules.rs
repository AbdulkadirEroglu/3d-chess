//! @file rules.rs
//! @brief Rules tests through the public API: perft move counts, special moves, rejections and results.

use chess_core::{
    Color, CommandSource, Game, GameResult, MoveCommand, MoveError, Piece, PieceKind, Square,
};

/// @brief Parses a square name; panics on typos so tests fail loudly.
fn sq(name: &str) -> Square {
    name.parse().expect("valid square")
}

/// @brief Builds a move command from square names.
fn cmd(from: &str, to: &str, promotion: Option<PieceKind>) -> MoveCommand {
    MoveCommand {
        from: sq(from),
        to: sq(to),
        promotion,
        source: CommandSource::Pointer,
    }
}

/// @brief Plays a list of moves like "e2e4", panicking if any is rejected.
fn play(game: &mut Game, moves: &[&str]) {
    for mv in moves {
        game.apply(cmd(&mv[0..2], &mv[2..4], None))
            .unwrap_or_else(|error| panic!("{mv} rejected: {error}"));
    }
}

/// @brief Counts leaf positions at a depth by playing every legal move through `apply`.
/// @note Standard perft: proves move generation and our move translation together.
fn perft(game: &Game, depth: u32) -> u64 {
    let moves = game.legal_moves();
    if depth == 1 {
        return moves.len() as u64;
    }
    moves
        .iter()
        .map(|mv| {
            let mut next = game.clone();
            next.apply(MoveCommand {
                from: mv.from,
                to: mv.to,
                promotion: mv.promotion,
                source: CommandSource::Ai,
            })
            .expect("legal move from legal_moves");
            perft(&next, depth - 1)
        })
        .sum()
}

#[test]
fn perft_start_position() {
    let game = Game::new();
    assert_eq!(perft(&game, 1), 20);
    assert_eq!(perft(&game, 2), 400);
    assert_eq!(perft(&game, 3), 8_902);
    assert_eq!(perft(&game, 4), 197_281);
}

#[test]
fn perft_kiwipete() {
    // Castling, en passant, promotions and pins all in one position.
    let game =
        Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .expect("valid fen");
    assert_eq!(perft(&game, 1), 48);
    assert_eq!(perft(&game, 2), 2_039);
    assert_eq!(perft(&game, 3), 97_862);
}

#[test]
fn perft_en_passant_pins() {
    let game = Game::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").expect("valid fen");
    assert_eq!(perft(&game, 1), 14);
    assert_eq!(perft(&game, 2), 191);
    assert_eq!(perft(&game, 3), 2_812);
    assert_eq!(perft(&game, 4), 43_238);
}

#[test]
fn squares_parse_and_print() {
    let e4 = sq("e4");
    assert_eq!((e4.file(), e4.rank()), (4, 3));
    assert_eq!(e4.to_string(), "e4");
    assert!("i9".parse::<Square>().is_err());
    assert!("e".parse::<Square>().is_err());
    assert_eq!(Square::new(8, 0), None);
}

#[test]
fn start_position_pieces() {
    let game = Game::new();
    assert_eq!(game.pieces().count(), 32);
    assert_eq!(
        game.piece_at(sq("e1")),
        Some(Piece {
            kind: PieceKind::King,
            color: Color::White
        })
    );
    assert_eq!(game.side_to_move(), Color::White);
    assert_eq!(game.result(), GameResult::Ongoing);
}

#[test]
fn castling_uses_king_target_square() {
    let mut game = Game::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid fen");
    let targets: Vec<_> = game
        .legal_moves()
        .into_iter()
        .filter(|mv| mv.from == sq("e1"))
        .map(|mv| mv.to.to_string())
        .collect();
    assert!(targets.contains(&"g1".to_string()));
    assert!(targets.contains(&"c1".to_string()));

    let mv = game.apply(cmd("e1", "g1", None)).expect("castle short");
    assert_eq!(mv.rook_move, Some((sq("h1"), sq("f1"))));
    assert_eq!(mv.capture, None);
    assert_eq!(
        game.piece_at(sq("g1")).map(|p| p.kind),
        Some(PieceKind::King)
    );
    assert_eq!(
        game.piece_at(sq("f1")).map(|p| p.kind),
        Some(PieceKind::Rook)
    );

    let mv = game.apply(cmd("e8", "c8", None)).expect("castle long");
    assert_eq!(mv.rook_move, Some((sq("a8"), sq("d8"))));
}

#[test]
fn en_passant_reports_captured_square() {
    let mut game = Game::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
        .expect("valid fen");
    let mv = game.apply(cmd("e5", "f6", None)).expect("en passant");
    assert_eq!(
        mv.capture,
        Some((
            Piece {
                kind: PieceKind::Pawn,
                color: Color::Black
            },
            sq("f5")
        ))
    );
    assert_eq!(game.piece_at(sq("f5")), None);
}

#[test]
fn promotion_needs_a_piece() {
    let mut game = Game::from_fen("8/P7/8/8/8/8/8/k6K w - - 0 1").expect("valid fen");
    assert_eq!(
        game.apply(cmd("a7", "a8", None)),
        Err(MoveError::PromotionRequired)
    );
    let mv = game
        .apply(cmd("a7", "a8", Some(PieceKind::Queen)))
        .expect("promote");
    assert_eq!(mv.promotion, Some(PieceKind::Queen));
    assert_eq!(
        game.piece_at(sq("a8")).map(|p| p.kind),
        Some(PieceKind::Queen)
    );
}

#[test]
fn bad_commands_are_rejected() {
    let mut game = Game::new();
    assert_eq!(
        game.apply(cmd("e4", "e5", None)),
        Err(MoveError::NoPiece(sq("e4")))
    );
    assert_eq!(
        game.apply(cmd("e7", "e5", None)),
        Err(MoveError::NotYourPiece(sq("e7")))
    );
    assert_eq!(game.apply(cmd("e2", "e5", None)), Err(MoveError::Illegal));
    // Rejected commands don't change the game.
    assert_eq!(game.side_to_move(), Color::White);
}

#[test]
fn checkmate_ends_the_game() {
    let mut game = Game::new();
    assert!(!game.in_check());
    play(&mut game, &["f2f3", "e7e5", "g2g4", "d8h4"]);
    assert!(game.in_check());
    assert_eq!(
        game.result(),
        GameResult::Checkmate {
            winner: Color::Black
        }
    );
    assert!(game.legal_moves().is_empty());
    assert_eq!(game.apply(cmd("a2", "a3", None)), Err(MoveError::GameOver));
}

#[test]
fn stalemate_is_detected() {
    let game = Game::from_fen("k7/8/1QK5/8/8/8/8/8 b - - 0 1").expect("valid fen");
    assert_eq!(game.result(), GameResult::Stalemate);
}

#[test]
fn fifty_move_rule_draws() {
    let mut game = Game::from_fen("k7/8/8/8/8/8/8/K6R w - - 99 80").expect("valid fen");
    play(&mut game, &["h1h2"]);
    assert_eq!(game.result(), GameResult::FiftyMoveRule);
}

#[test]
fn threefold_repetition_draws() {
    let mut game = Game::new();
    let shuffle = ["g1f3", "g8f6", "f3g1", "f6g8"];
    play(&mut game, &shuffle);
    assert_eq!(game.result(), GameResult::Ongoing);
    play(&mut game, &shuffle);
    assert_eq!(game.result(), GameResult::ThreefoldRepetition);
}

#[test]
fn insufficient_material_draws() {
    // King takes the last pawn: king against king.
    let mut game = Game::from_fen("k7/8/8/8/8/8/1p6/K7 w - - 0 1").expect("valid fen");
    assert_eq!(game.result(), GameResult::Ongoing);
    play(&mut game, &["a1b2"]);
    assert_eq!(game.result(), GameResult::InsufficientMaterial);

    // Bishops on the same square color can never mate.
    let same = Game::from_fen("k7/8/8/8/8/8/8/KB5b w - - 0 1").expect("valid fen");
    assert_eq!(same.result(), GameResult::InsufficientMaterial);

    // Bishops on opposite colors still can.
    let opposite = Game::from_fen("kb6/8/8/8/8/8/8/KB6 w - - 0 1").expect("valid fen");
    assert_eq!(opposite.result(), GameResult::Ongoing);
}
