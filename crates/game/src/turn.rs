//! @file turn.rs
//! @brief Owns the active game. Turn flow (sending move commands to `chess_core`) arrives in step 5.

use bevy::prelude::*;
use chess_core::Game;

/// @brief The game being played: the single source of truth for every piece and square.
/// @note The 3D scene only shows this; never read game state back from entities.
#[derive(Resource, Default)]
pub struct ActiveGame(pub Game);

/// @brief Sets up the active game.
pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveGame>();
    }
}
