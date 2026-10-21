//! @file test_support.rs
//! @brief Shared helpers for headless game tests: an app without a window, and playing moves.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use chess_core::{CommandSource, Game, MoveCommand, PieceKind, Square};

use crate::animation::{AnimationPlugin, AnimationQueue};
use crate::pieces::PiecesPlugin;
use crate::turn::{ActiveGame, MoveCommandSent, TurnPlugin};

/// @brief Headless app with rules, pieces and animations, starting from `game`.
/// @note Time advances a fixed 100 ms per update, so animation tests are deterministic.
pub fn headless_app(game: Game) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .insert_resource(ActiveGame(game))
        .add_plugins((TurnPlugin, PiecesPlugin, AnimationPlugin));
    app.update();
    app
}

/// @brief Sends a move like "e2e4" (or "a7a8q" to promote) and runs until its animations finish.
pub fn play(app: &mut App, mv: &str) {
    let square = |name: &str| name.parse::<Square>().expect("valid square");
    app.world_mut().write_message(MoveCommandSent(MoveCommand {
        from: square(&mv[0..2]),
        to: square(&mv[2..4]),
        promotion: (mv.len() == 5).then_some(PieceKind::Queen),
        source: CommandSource::Pointer,
    }));
    app.update();
    for _ in 0..50 {
        if !app.world().resource::<AnimationQueue>().is_busy() {
            return;
        }
        app.update();
    }
    panic!("animations for {mv} never finished");
}
