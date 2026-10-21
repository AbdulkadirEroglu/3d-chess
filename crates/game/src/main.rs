//! @file main.rs
//! @brief Entry point: builds the Bevy app and registers the game plugins.

mod ai;
mod animation;
mod board;
mod camera;
mod input;
mod menu;
mod pieces;
mod turn;
mod ui;

#[cfg(test)]
mod test_support;

use bevy::log::LogPlugin;
use bevy::prelude::*;

/// @brief Dev log filter: our crates at debug, everything else at info.
const DEV_LOG_FILTER: &str = "info,game=debug,chess_core=debug,chess_ai=debug,wgpu=error,naga=warn";

/// @brief Release log filter: info only.
const RELEASE_LOG_FILTER: &str = "info,wgpu=error,naga=warn";

/// @brief Starts the game.
/// @note `RUST_LOG` overrides the log filter.
fn main() {
    let filter = if cfg!(debug_assertions) {
        DEV_LOG_FILTER
    } else {
        RELEASE_LOG_FILTER
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: filter.to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "3D Chess".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            turn::TurnPlugin,
            board::BoardPlugin,
            camera::CameraPlugin,
            pieces::PiecesPlugin,
            input::InputPlugin,
            animation::AnimationPlugin,
            ui::UiPlugin,
            ai::AiPlugin,
            menu::MenuPlugin,
        ))
        .add_systems(Startup, log_startup)
        .run();
}

/// @brief Logs the game version once at startup.
fn log_startup() {
    info!(version = env!("CARGO_PKG_VERSION"), "game started");
}
