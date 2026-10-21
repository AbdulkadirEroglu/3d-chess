//! @file ui.rs
//! @brief On-screen UI: status line (turn, check, AI), promotion piece picker, game-over banner.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::picking::Pickable;
use bevy::prelude::*;
use chess_core::{Color as Side, Game, GameResult, PieceKind};

use crate::ai::AiPlayer;
use crate::animation::AnimationQueue;
use crate::input::Selection;
use crate::menu::AppState;
use crate::turn::{ActiveGame, GameSet};

/// @brief Size of the status line at the top.
const STATUS_FONT_SIZE: f32 = 26.0;

/// @brief Size of the game-over banner.
const BANNER_FONT_SIZE: f32 = 40.0;

/// @brief Size of the picker's title and button labels.
const PICKER_FONT_SIZE: f32 = 26.0;

/// @brief Text color: cream, like the white pieces.
pub const TEXT_COLOR: Color = Color::srgb(0.95, 0.92, 0.85);

/// @brief Panel background (banner, picker, menu): dark and slightly see-through.
pub const PANEL_BACKGROUND: Color = Color::srgba(0.08, 0.08, 0.10, 0.88);

/// @brief Button background: dark stone with a gold tint.
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.30, 0.25, 0.17);

/// @brief Spawns the UI and keeps it in sync with the game.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui).add_systems(
            Update,
            (update_ui, update_promotion_picker).after(GameSet::Animation),
        );
    }
}

/// @brief Marks the status line text.
#[derive(Component)]
struct StatusText;

/// @brief Marks the full-screen container that centers the banner; hidden while the game runs.
#[derive(Component)]
struct Banner;

/// @brief Marks the banner text.
#[derive(Component)]
struct BannerText;

/// @brief Marks the full-screen container of the promotion picker; shown while a promotion waits.
#[derive(Component)]
struct PromotionPicker;

/// @brief A promotion button and the piece it picks. Input reads clicks on it.
#[derive(Component, Debug)]
pub struct PromotionChoice(pub PieceKind);

/// @brief Full-screen layer that centers its child and never catches clicks itself.
pub fn centered_layer() -> (Node, Visibility, Pickable) {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        Pickable::IGNORE,
    )
}

/// @brief Spawns a clickable button with a text label; `marker` identifies what it does.
/// @note The label ignores picking, so clicks land on the button entity itself.
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    label: &str,
    font_size: f32,
) {
    parent
        .spawn((
            marker,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BACKGROUND),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont::from_font_size(font_size),
                TextColor(TEXT_COLOR),
                Pickable::IGNORE,
            ));
        });
}

/// @brief Spawns the status line, and the hidden game-over banner and promotion picker.
fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        StatusText,
        Text::new(""),
        TextFont::from_font_size(STATUS_FONT_SIZE),
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));

    commands
        .spawn((Banner, centered_layer()))
        .with_children(|parent| {
            parent.spawn((
                BannerText,
                Text::new(""),
                TextFont::from_font_size(BANNER_FONT_SIZE),
                TextColor(TEXT_COLOR),
                TextLayout::justify(Justify::Center),
                Node {
                    padding: UiRect::axes(Val::Px(40.0), Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(PANEL_BACKGROUND),
            ));
        });

    commands
        .spawn((PromotionPicker, centered_layer()))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BACKGROUND),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Promote to"),
                        TextFont::from_font_size(PICKER_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                        Pickable::IGNORE,
                    ));
                    panel
                        .spawn(Node {
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for (kind, label) in [
                                (PieceKind::Queen, "Queen"),
                                (PieceKind::Rook, "Rook"),
                                (PieceKind::Bishop, "Bishop"),
                                (PieceKind::Knight, "Knight"),
                            ] {
                                spawn_button(row, PromotionChoice(kind), label, PICKER_FONT_SIZE);
                            }
                        });
                });
        });
}

/// @brief Updates the status line, and shows the banner once the game is over and the board is still.
/// @note Text is only written when it changes, so the layout isn't redone every frame.
fn update_ui(
    state: Res<State<AppState>>,
    game: Res<ActiveGame>,
    ai: Res<AiPlayer>,
    queue: Res<AnimationQueue>,
    mut status: Query<&mut Text, (With<StatusText>, Without<BannerText>)>,
    mut banner_text: Query<&mut Text, (With<BannerText>, Without<StatusText>)>,
    mut banner: Query<&mut Visibility, With<Banner>>,
) {
    // The menu covers the board, so the HUD stays empty there.
    let in_menu = *state.get() == AppState::Menu;
    let status_line = if in_menu {
        String::new()
    } else {
        status_text(&game.0, &ai)
    };
    for mut text in &mut status {
        if text.0 != status_line {
            text.0 = status_line.clone();
        }
    }

    let message = if in_menu || queue.is_busy() {
        None
    } else {
        banner_message(game.0.result())
    };
    for mut text in &mut banner_text {
        if let Some(message) = &message
            && &text.0 != message
        {
            debug!(%message, "game over banner shown");
            text.0 = message.clone();
        }
    }
    set_visible(&mut banner, message.is_some());
}

/// @brief Shows the promotion picker while a promotion waits for the player's choice.
fn update_promotion_picker(
    selection: Res<Selection>,
    mut picker: Query<&mut Visibility, With<PromotionPicker>>,
) {
    set_visible(&mut picker, selection.pending_promotion.is_some());
}

/// @brief Shows or hides UI layers, touching them only when the state actually changes.
pub fn set_visible<F: bevy::ecs::query::QueryFilter>(
    layers: &mut Query<&mut Visibility, F>,
    visible: bool,
) {
    let wanted = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in layers.iter_mut() {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

/// @brief Status line: whose turn it is, whether that's the AI, and whether they're in check.
fn status_text(game: &Game, ai: &AiPlayer) -> String {
    if game.result() != GameResult::Ongoing {
        return "Game over".to_string();
    }
    let mover = game.side_to_move();
    let side = if ai.plays(mover) {
        format!("{} (AI)", side_name(mover))
    } else {
        side_name(mover).to_string()
    };
    if game.in_check() {
        format!("{side} to move - check!")
    } else {
        format!("{side} to move")
    }
}

/// @brief Banner text for a finished game; None while it's still going.
fn banner_message(result: GameResult) -> Option<String> {
    let headline = match result {
        GameResult::Ongoing => return None,
        GameResult::Checkmate { winner } => format!("Checkmate - {} wins", side_name(winner)),
        GameResult::Stalemate => "Draw - stalemate".to_string(),
        GameResult::FiftyMoveRule => "Draw - 50-move rule".to_string(),
        GameResult::ThreefoldRepetition => "Draw - threefold repetition".to_string(),
        GameResult::InsufficientMaterial => "Draw - insufficient material".to_string(),
    };
    Some(format!(
        "{headline}\nPress R to play again, Esc for the menu"
    ))
}

/// @brief Display name of a side.
fn side_name(side: Side) -> &'static str {
    match side {
        Side::White => "White",
        Side::Black => "Black",
    }
}

#[cfg(test)]
mod tests {
    use chess_core::{CommandSource, MoveCommand};

    use super::*;

    /// @brief Two human players, no AI.
    fn humans() -> AiPlayer {
        AiPlayer {
            white: false,
            black: false,
            depth: 1,
        }
    }

    /// @brief Plays moves like "e2e4" on a game.
    fn play(game: &mut Game, moves: &[&str]) {
        for mv in moves {
            game.apply(MoveCommand {
                from: mv[0..2].parse().expect("square"),
                to: mv[2..4].parse().expect("square"),
                promotion: None,
                source: CommandSource::Pointer,
            })
            .expect("legal move");
        }
    }

    /// @brief The status line follows turns, check and the AI's side.
    #[test]
    fn status_follows_turn_and_check() {
        let mut game = Game::new();
        assert_eq!(status_text(&game, &humans()), "White to move");
        play(&mut game, &["e2e4", "f7f6", "d1h5"]);
        assert_eq!(status_text(&game, &humans()), "Black to move - check!");
        assert_eq!(
            status_text(&game, &AiPlayer::default()),
            "Black (AI) to move - check!"
        );
    }

    /// @brief Checkmate and draws produce a banner; an ongoing game doesn't.
    #[test]
    fn banner_for_finished_games() {
        let mut game = Game::new();
        assert_eq!(banner_message(game.result()), None);
        play(&mut game, &["f2f3", "e7e5", "g2g4", "d8h4"]);
        assert_eq!(status_text(&game, &humans()), "Game over");
        assert_eq!(
            banner_message(game.result()).as_deref(),
            Some("Checkmate - Black wins\nPress R to play again, Esc for the menu")
        );
        assert_eq!(
            banner_message(GameResult::Stalemate).as_deref(),
            Some("Draw - stalemate\nPress R to play again, Esc for the menu")
        );
    }
}
