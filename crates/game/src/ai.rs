//! @file ai.rs
//! @brief Runs the computer opponent on a background thread and sends its moves as `MoveCommand`s.
//! @note The AI is just another input source: the turn flow applies its moves like any click.

use bevy::prelude::*;
use bevy::tasks::futures::check_ready;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use chess_core::{Color as Side, CommandSource, GameResult, Move, MoveCommand};

use crate::animation::AnimationQueue;
use crate::turn::{ActiveGame, GameSet, GameStarted, MoveCommandSent};

/// @brief How many plies (half-moves) the AI looks ahead. Depth 4 answers in milliseconds.
const AI_DEPTH: u32 = 4;

/// @brief Starts AI searches on its turns and sends the chosen moves.
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiPlayer>()
            .init_resource::<AiThinking>()
            .add_systems(
                Update,
                (cancel_on_new_game, start_ai_turn, finish_ai_turn)
                    .chain()
                    .in_set(GameSet::Input),
            );
    }
}

/// @brief Which side the AI plays, and how deep it searches.
/// @note Black by default until the mode select menu (alpha step 7) lets players choose.
#[derive(Resource, Debug)]
pub struct AiPlayer {
    /// @brief The AI's side; None for two human players.
    pub side: Option<Side>,
    /// @brief Search depth in plies.
    pub depth: u32,
}

impl Default for AiPlayer {
    fn default() -> Self {
        Self {
            side: Some(Side::Black),
            depth: AI_DEPTH,
        }
    }
}

/// @brief The running search, if the AI is thinking.
#[derive(Resource, Default)]
struct AiThinking(Option<Task<Option<Move>>>);

/// @brief Drops a running search when a new game starts; its move would belong to the old game.
fn cancel_on_new_game(mut started: MessageReader<GameStarted>, mut thinking: ResMut<AiThinking>) {
    if started.read().count() > 0 && thinking.0.take().is_some() {
        debug!("AI search cancelled by new game");
    }
}

/// @brief Starts a search when it's the AI's turn and the board has stopped moving.
fn start_ai_turn(
    ai: Res<AiPlayer>,
    game: Res<ActiveGame>,
    queue: Res<AnimationQueue>,
    mut thinking: ResMut<AiThinking>,
) {
    let ai_to_move =
        ai.side == Some(game.0.side_to_move()) && game.0.result() == GameResult::Ongoing;
    if !ai_to_move || queue.is_busy() || thinking.0.is_some() {
        return;
    }
    let position = game.0.clone();
    let depth = ai.depth;
    thinking.0 = Some(
        AsyncComputeTaskPool::get().spawn(async move { chess_ai::choose_move(&position, depth) }),
    );
    debug!(depth, "AI thinking");
}

/// @brief Sends the AI's move once its search has finished.
fn finish_ai_turn(mut thinking: ResMut<AiThinking>, mut sent: MessageWriter<MoveCommandSent>) {
    let Some(task) = thinking.0.as_mut() else {
        return;
    };
    let Some(result) = check_ready(task) else {
        return;
    };
    thinking.0 = None;
    match result {
        Some(mv) => {
            debug!(from = %mv.from, to = %mv.to, "AI chose a move");
            sent.write(MoveCommandSent(MoveCommand {
                from: mv.from,
                to: mv.to,
                promotion: mv.promotion,
                source: CommandSource::Ai,
            }));
        }
        None => warn!("AI found no move in an ongoing game"),
    }
}

#[cfg(test)]
mod tests {
    use chess_core::Game;

    use super::*;
    use crate::test_support::{headless_app, play};

    /// @brief Runs the app until it's `side`'s turn again, or fails after a few seconds.
    fn wait_for_turn(app: &mut App, side: Side) {
        for _ in 0..500 {
            if app.world().resource::<ActiveGame>().0.side_to_move() == side
                && !app.world().resource::<AnimationQueue>().is_busy()
            {
                return;
            }
            app.update();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the AI never moved");
    }

    /// @brief After a human move, the AI (Black) answers with a legal move of its own.
    #[test]
    fn ai_answers_as_black() {
        let mut app = headless_app(Game::new());
        app.add_plugins(AiPlugin);
        play(&mut app, "e2e4");
        wait_for_turn(&mut app, Side::White);

        let game = &app.world().resource::<ActiveGame>().0;
        assert_eq!(game.side_to_move(), Side::White);
        assert_eq!(game.pieces().count(), 32);
    }
}
