//! @file turn.rs
//! @brief Turn flow: owns the active game, applies move commands from any source, and announces played moves.

use bevy::prelude::*;
use chess_core::{Game, GameResult, Move, MoveCommand};

/// @brief The game being played: the single source of truth for every piece and square.
/// @note The 3D scene only shows this; never read game state back from entities.
#[derive(Resource, Default)]
pub struct ActiveGame(pub Game);

/// @brief Per-frame order: input sends commands, the turn flow applies them, animations show them.
/// @note All in one frame, so the animation queue is busy before the next click can be read.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSet {
    /// @brief Input sources produce `MoveCommandSent`.
    Input,
    /// @brief The rules apply commands and send `MovePlayed`.
    Turn,
    /// @brief The animation queue picks up played moves and plays them.
    Animation,
}

/// @brief A move command from any input source, waiting for the turn flow to apply it.
#[derive(Message, Debug, Clone, Copy)]
pub struct MoveCommandSent(pub MoveCommand);

/// @brief Asks for a fresh game. Input sends it; the turn flow resets the game.
#[derive(Message, Debug, Clone, Copy)]
pub struct NewGameRequested;

/// @brief A game has begun (at startup or after a restart). Pieces respawn from `ActiveGame`.
#[derive(Message, Debug, Clone, Copy)]
pub struct GameStarted;

/// @brief A move the rules accepted and played. The animation queue shows it.
#[derive(Message, Debug, Clone, Copy)]
pub struct MovePlayed(pub Move);

/// @brief Sets up the active game and applies move commands.
pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveGame>()
            .add_message::<MoveCommandSent>()
            .add_message::<MovePlayed>()
            .add_message::<NewGameRequested>()
            .add_message::<GameStarted>()
            .configure_sets(
                Update,
                (GameSet::Input, GameSet::Turn, GameSet::Animation).chain(),
            )
            .add_systems(Startup, announce_first_game)
            .add_systems(
                Update,
                (start_new_game, apply_commands).in_set(GameSet::Turn),
            );
    }
}

/// @brief Announces the game created at startup, so the pieces spawn.
fn announce_first_game(mut started: MessageWriter<GameStarted>) {
    started.write(GameStarted);
}

/// @brief Replaces the game with a fresh one when a new game is requested.
fn start_new_game(
    mut requests: MessageReader<NewGameRequested>,
    mut game: ResMut<ActiveGame>,
    mut started: MessageWriter<GameStarted>,
) {
    if requests.read().count() == 0 {
        return;
    }
    *game = ActiveGame::default();
    info!("new game started");
    started.write(GameStarted);
}

/// @brief Applies each move command to the game and announces the moves that were played.
/// @note Rejected commands are logged by `chess_core`; nothing changes for them.
fn apply_commands(
    mut sent: MessageReader<MoveCommandSent>,
    mut game: ResMut<ActiveGame>,
    mut played: MessageWriter<MovePlayed>,
) {
    for MoveCommandSent(cmd) in sent.read() {
        if let Ok(mv) = game.0.apply(*cmd) {
            info!(
                from = %mv.from,
                to = %mv.to,
                piece = ?mv.piece,
                capture = ?mv.capture.map(|(piece, _)| piece.kind),
                source = ?cmd.source,
                "move played"
            );
            played.write(MovePlayed(mv));
            if game.0.result() != GameResult::Ongoing {
                info!(result = ?game.0.result(), "game over");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chess_core::{Color, CommandSource, Square};

    use super::*;

    /// @brief Builds a pointer command from square names.
    fn cmd(from: &str, to: &str) -> MoveCommandSent {
        let square = |name: &str| name.parse::<Square>().expect("valid square");
        MoveCommandSent(MoveCommand {
            from: square(from),
            to: square(to),
            promotion: None,
            source: CommandSource::Pointer,
        })
    }

    /// @brief Illegal commands change nothing, legal ones are played and announced,
    ///        and a new game request resets the game and announces the start.
    #[test]
    fn commands_and_new_game() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TurnPlugin));
        let side = |app: &App| app.world().resource::<ActiveGame>().0.side_to_move();

        app.world_mut().write_message(cmd("e2", "e5"));
        app.update();
        assert_eq!(side(&app), Color::White);
        assert_eq!(app.world().resource::<Messages<MovePlayed>>().len(), 0);

        app.world_mut().write_message(cmd("e2", "e4"));
        app.update();
        assert_eq!(side(&app), Color::Black);
        assert_eq!(app.world().resource::<Messages<MovePlayed>>().len(), 1);

        // Count only the announcements sent after the request.
        let mut cursor = app
            .world()
            .resource::<Messages<GameStarted>>()
            .get_cursor_current();
        app.world_mut().write_message(NewGameRequested);
        app.update();
        assert_eq!(side(&app), Color::White);
        let started = app.world().resource::<Messages<GameStarted>>();
        assert_eq!(cursor.read(started).count(), 1);
    }
}
