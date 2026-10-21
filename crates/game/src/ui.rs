//! @file ui.rs
//! @brief On-screen text: whose turn it is, check, and the game-over banner.

use bevy::picking::Pickable;
use bevy::prelude::*;
use chess_core::{Color as Side, Game, GameResult};

use crate::animation::AnimationQueue;
use crate::turn::{ActiveGame, GameSet};

/// @brief Size of the status line at the top.
const STATUS_FONT_SIZE: f32 = 26.0;

/// @brief Size of the game-over banner.
const BANNER_FONT_SIZE: f32 = 40.0;

/// @brief Text color: cream, like the white pieces.
const TEXT_COLOR: Color = Color::srgb(0.95, 0.92, 0.85);

/// @brief Banner background: dark and slightly see-through.
const BANNER_BACKGROUND: Color = Color::srgba(0.08, 0.08, 0.10, 0.88);

/// @brief Spawns the text and keeps it in sync with the game.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui)
            .add_systems(Update, update_ui.after(GameSet::Animation));
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

/// @brief Spawns the status line and the (hidden) game-over banner.
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
        .spawn((
            Banner,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            // Full-screen, so it must never swallow clicks meant for the board.
            Pickable::IGNORE,
        ))
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
                BackgroundColor(BANNER_BACKGROUND),
            ));
        });
}

/// @brief Updates the status line, and shows the banner once the game is over and the board is still.
/// @note Text is only written when it changes, so the layout isn't redone every frame.
fn update_ui(
    game: Res<ActiveGame>,
    queue: Res<AnimationQueue>,
    mut status: Query<&mut Text, (With<StatusText>, Without<BannerText>)>,
    mut banner_text: Query<&mut Text, (With<BannerText>, Without<StatusText>)>,
    mut banner: Query<&mut Visibility, With<Banner>>,
) {
    let status_line = status_text(&game.0);
    for mut text in &mut status {
        if text.0 != status_line {
            text.0 = status_line.clone();
        }
    }

    let message = if queue.is_busy() {
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
    let wanted = if message.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut banner {
        if *visibility != wanted {
            *visibility = wanted;
        }
    }
}

/// @brief Status line: whose turn it is and whether they're in check.
fn status_text(game: &Game) -> String {
    if game.result() != GameResult::Ongoing {
        return "Game over".to_string();
    }
    let side = side_name(game.side_to_move());
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
    Some(format!("{headline}\nPress R to play again"))
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

    /// @brief The status line follows turns and check.
    #[test]
    fn status_follows_turn_and_check() {
        let mut game = Game::new();
        assert_eq!(status_text(&game), "White to move");
        play(&mut game, &["e2e4", "f7f6", "d1h5"]);
        assert_eq!(status_text(&game), "Black to move - check!");
    }

    /// @brief Checkmate and draws produce a banner; an ongoing game doesn't.
    #[test]
    fn banner_for_finished_games() {
        let mut game = Game::new();
        assert_eq!(banner_message(game.result()), None);
        play(&mut game, &["f2f3", "e7e5", "g2g4", "d8h4"]);
        assert_eq!(status_text(&game), "Game over");
        assert_eq!(
            banner_message(game.result()).as_deref(),
            Some("Checkmate - Black wins\nPress R to play again")
        );
        assert_eq!(
            banner_message(GameResult::Stalemate).as_deref(),
            Some("Draw - stalemate\nPress R to play again")
        );
    }
}
