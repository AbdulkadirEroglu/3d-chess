//! @file input.rs
//! @brief Mouse/touch input: click a piece to select it, click a highlighted square to send a `MoveCommand`.
//!        Promotions open a piece picker. After the game ends, R asks for a new game.

use bevy::picking::events::PointerClick;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use chess_core::{CommandSource, Game, GameResult, Move, MoveCommand, PieceKind, Square};

use crate::ai::AiPlayer;
use crate::animation::AnimationQueue;
use crate::board::world_to_square;
use crate::menu::AppState;
use crate::turn::{ActiveGame, GameSet, MoveCommandSent, NewGameRequested};
use crate::ui::PromotionChoice;

/// @brief Turns clicks on squares, pieces and promotion buttons into selections and move commands.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .init_resource::<Selection>()
            .add_systems(
                Update,
                (handle_clicks, request_new_game)
                    .run_if(in_state(AppState::Playing))
                    .in_set(GameSet::Input),
            );
    }
}

/// @brief The selected square and its legal moves. Empty when nothing is selected.
#[derive(Resource, Default, Debug, PartialEq)]
pub struct Selection {
    /// @brief Square of the selected piece.
    pub square: Option<Square>,
    /// @brief Legal moves of the selected piece; their targets get highlighted.
    pub moves: Vec<Move>,
    /// @brief A pawn move waiting for the player to pick its promotion piece: (from, to).
    pub pending_promotion: Option<(Square, Square)>,
}

/// @brief Reads left clicks on squares, pieces and promotion buttons and reacts to them.
fn handle_clicks(
    mut clicks: MessageReader<PointerClick>,
    choices: Query<&PromotionChoice>,
    game: Res<ActiveGame>,
    queue: Res<AnimationQueue>,
    ai: Res<AiPlayer>,
    mut selection: ResMut<Selection>,
    mut sent: MessageWriter<MoveCommandSent>,
) {
    // The board is locked while pieces move and while the AI has the turn.
    if queue.is_busy() || ai.side == Some(game.0.side_to_move()) {
        clicks.clear();
        return;
    }
    for click in clicks.read() {
        if click.button != PointerButton::Primary {
            continue;
        }
        if let Ok(PromotionChoice(kind)) = choices.get(click.entity) {
            if let Some(cmd) = choose_promotion(&mut selection, *kind) {
                sent.write(MoveCommandSent(cmd));
            }
            continue;
        }
        // The square under the clicked point: works for tiles, placeholders and any model.
        let Some(square) = click.hit.position.and_then(world_to_square) else {
            continue;
        };
        if let Some(cmd) = on_square_clicked(&mut selection, &game.0, square) {
            sent.write(MoveCommandSent(cmd));
        }
    }
}

/// @brief Asks for a new game when R is pressed, once the game has ended and the board is still.
fn request_new_game(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<ActiveGame>,
    queue: Res<AnimationQueue>,
    mut requests: MessageWriter<NewGameRequested>,
) {
    let finished = game.0.result() != GameResult::Ongoing && !queue.is_busy();
    if finished && keys.just_pressed(KeyCode::KeyR) {
        debug!("new game requested");
        requests.write(NewGameRequested);
    }
}

/// @brief Completes a waiting promotion with the piece the player picked.
/// @return The move command, or None if no promotion was waiting.
fn choose_promotion(selection: &mut Selection, kind: PieceKind) -> Option<MoveCommand> {
    let (from, to) = selection.pending_promotion?;
    let cmd = MoveCommand {
        from,
        to,
        promotion: Some(kind),
        source: CommandSource::Pointer,
    };
    debug!(?cmd, "promotion chosen, move command sent");
    *selection = Selection::default();
    Some(cmd)
}

/// @brief Applies the click rules to one clicked square.
/// @return A move command when the click completes a move.
fn on_square_clicked(
    selection: &mut Selection,
    game: &Game,
    square: Square,
) -> Option<MoveCommand> {
    // While the promotion picker is open, a board click cancels the move.
    if selection.pending_promotion.is_some() {
        debug!("promotion cancelled");
        *selection = Selection::default();
        return None;
    }

    // A click on a highlighted target plays the move, or asks for a promotion piece first.
    if let Some(mv) = selection.moves.iter().find(|mv| mv.to == square) {
        if mv.promotion.is_some() {
            debug!(from = %mv.from, to = %mv.to, "promotion piece needed");
            selection.pending_promotion = Some((mv.from, mv.to));
            return None;
        }
        let cmd = MoveCommand {
            from: mv.from,
            to: mv.to,
            promotion: None,
            source: CommandSource::Pointer,
        };
        debug!(?cmd, "move command sent");
        *selection = Selection::default();
        return Some(cmd);
    }

    // A click on one of your own pieces selects it; clicking it again, or anything else, clears.
    let own_piece = game
        .piece_at(square)
        .is_some_and(|piece| piece.color == game.side_to_move());
    if own_piece && selection.square != Some(square) {
        let moves = game
            .legal_moves()
            .into_iter()
            .filter(|mv| mv.from == square)
            .collect();
        debug!(%square, "piece selected");
        *selection = Selection {
            square: Some(square),
            moves,
            pending_promotion: None,
        };
    } else if selection.square.is_some() {
        debug!("selection cleared");
        *selection = Selection::default();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @brief Parses a square name for tests.
    fn sq(name: &str) -> Square {
        name.parse().expect("valid square")
    }

    /// @brief Selecting a piece, then clicking a target, produces the move command.
    #[test]
    fn select_then_move() {
        let game = Game::new();
        let mut selection = Selection::default();

        assert_eq!(on_square_clicked(&mut selection, &game, sq("e2")), None);
        assert_eq!(selection.square, Some(sq("e2")));
        let mut targets: Vec<_> = selection.moves.iter().map(|mv| mv.to.to_string()).collect();
        targets.sort();
        assert_eq!(targets, ["e3", "e4"]);

        let cmd = on_square_clicked(&mut selection, &game, sq("e4")).expect("move sent");
        assert_eq!(
            (cmd.from, cmd.to, cmd.promotion),
            (sq("e2"), sq("e4"), None)
        );
        assert_eq!(selection, Selection::default());
    }

    /// @brief Opponent pieces can't be selected, and clicking the selection again clears it.
    #[test]
    fn selection_rules() {
        let game = Game::new();
        let mut selection = Selection::default();

        on_square_clicked(&mut selection, &game, sq("e7"));
        assert_eq!(selection.square, None);

        on_square_clicked(&mut selection, &game, sq("g1"));
        assert_eq!(selection.square, Some(sq("g1")));
        on_square_clicked(&mut selection, &game, sq("g1"));
        assert_eq!(selection, Selection::default());

        // An empty, non-target square clears too.
        on_square_clicked(&mut selection, &game, sq("g1"));
        on_square_clicked(&mut selection, &game, sq("d5"));
        assert_eq!(selection, Selection::default());
    }

    /// @brief A pawn reaching the last rank waits for a piece choice; a board click cancels instead.
    #[test]
    fn promotion_waits_for_a_choice() {
        let game = Game::from_fen("8/P7/8/8/8/8/8/k6K w - - 0 1").expect("valid fen");
        let mut selection = Selection::default();

        on_square_clicked(&mut selection, &game, sq("a7"));
        assert_eq!(on_square_clicked(&mut selection, &game, sq("a8")), None);
        assert_eq!(selection.pending_promotion, Some((sq("a7"), sq("a8"))));
        let cmd = choose_promotion(&mut selection, PieceKind::Knight).expect("move sent");
        assert_eq!(
            (cmd.from, cmd.to, cmd.promotion),
            (sq("a7"), sq("a8"), Some(PieceKind::Knight))
        );
        assert_eq!(selection, Selection::default());

        on_square_clicked(&mut selection, &game, sq("a7"));
        on_square_clicked(&mut selection, &game, sq("a8"));
        on_square_clicked(&mut selection, &game, sq("d4"));
        assert_eq!(selection, Selection::default());
        assert_eq!(choose_promotion(&mut selection, PieceKind::Queen), None);
    }
}
