//! @file input.rs
//! @brief Mouse/touch input: click a piece to select it, click a highlighted square to send a `MoveCommand`.
//!        After the game ends, R asks for a new game.

use bevy::picking::events::PointerClick;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use chess_core::{CommandSource, Game, GameResult, Move, MoveCommand, PieceKind, Square};

use crate::animation::AnimationQueue;
use crate::board::BoardSquare;
use crate::pieces::PieceOnBoard;
use crate::turn::{ActiveGame, GameSet, MoveCommandSent, NewGameRequested};

/// @brief Turns clicks on squares and pieces into selections and move commands.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .init_resource::<Selection>()
            .add_systems(
                Update,
                (handle_clicks, request_new_game).in_set(GameSet::Input),
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
}

/// @brief Reads left clicks on board squares and pieces and reacts to the clicked square.
fn handle_clicks(
    mut clicks: MessageReader<PointerClick>,
    squares: Query<&BoardSquare>,
    pieces: Query<&PieceOnBoard>,
    game: Res<ActiveGame>,
    queue: Res<AnimationQueue>,
    mut selection: ResMut<Selection>,
    mut sent: MessageWriter<MoveCommandSent>,
) {
    // The board is locked until the last move has finished animating.
    if queue.is_busy() {
        clicks.clear();
        return;
    }
    for click in clicks.read() {
        if click.button != PointerButton::Primary {
            continue;
        }
        // Clicking a piece counts as clicking the square it stands on.
        let square = squares
            .get(click.entity)
            .map(|board_square| board_square.0)
            .or_else(|_| pieces.get(click.entity).map(|on_board| on_board.square));
        let Ok(square) = square else {
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

/// @brief Applies the click rules to one clicked square.
/// @return A move command when the click completes a move.
/// @note Promotions always become a queen in the alpha; a choice UI comes in beta.
fn on_square_clicked(
    selection: &mut Selection,
    game: &Game,
    square: Square,
) -> Option<MoveCommand> {
    // A click on a highlighted target plays the move.
    if let Some(mv) = selection.moves.iter().find(|mv| mv.to == square) {
        let cmd = MoveCommand {
            from: mv.from,
            to: mv.to,
            promotion: mv.promotion.map(|_| PieceKind::Queen),
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

    /// @brief A pawn reaching the last rank promotes to a queen.
    #[test]
    fn promotion_becomes_queen() {
        let game = Game::from_fen("8/P7/8/8/8/8/8/k6K w - - 0 1").expect("valid fen");
        let mut selection = Selection::default();
        on_square_clicked(&mut selection, &game, sq("a7"));
        let cmd = on_square_clicked(&mut selection, &game, sq("a8")).expect("move sent");
        assert_eq!(cmd.promotion, Some(PieceKind::Queen));
    }
}
