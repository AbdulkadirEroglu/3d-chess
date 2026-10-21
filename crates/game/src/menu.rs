//! @file menu.rs
//! @brief Main menu and app flow: pick opponent, side and AI level, then start. Esc returns to the menu.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::picking::Pickable;
use bevy::picking::events::PointerClick;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use chess_core::Color as Side;

use crate::ai::AiPlayer;
use crate::input::Selection;
use crate::turn::{GameSet, NewGameRequested};
use crate::ui::{
    BUTTON_BACKGROUND, PANEL_BACKGROUND, TEXT_COLOR, centered_layer, set_visible, spawn_button,
};

/// @brief Size of the menu title.
const TITLE_FONT_SIZE: f32 = 48.0;

/// @brief Size of menu labels and buttons.
const MENU_FONT_SIZE: f32 = 24.0;

/// @brief Size of the controls hint at the bottom of the menu.
const HINT_FONT_SIZE: f32 = 16.0;

/// @brief Hint text color: muted cream.
const HINT_COLOR: Color = Color::srgb(0.65, 0.62, 0.58);

/// @brief Background of the selected option, and of the Start button: gold.
const SELECTED_BACKGROUND: Color = Color::srgb(0.62, 0.46, 0.18);

/// @brief Width of the row labels, so the buttons line up.
const LABEL_WIDTH: f32 = 130.0;

/// @brief Main menu, settings and the switch between menu and play.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_resource::<MatchSettings>()
            .add_systems(Startup, spawn_menu)
            .add_systems(
                Update,
                (
                    handle_menu_clicks
                        .run_if(in_state(AppState::Menu))
                        .in_set(GameSet::Input),
                    open_menu_on_escape
                        .run_if(in_state(AppState::Playing))
                        .in_set(GameSet::Input),
                    update_menu_buttons,
                    sync_menu_visibility,
                ),
            );
    }
}

/// @brief Top-level app flow. The game starts in the menu, with the board visible behind it.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// @brief The main menu is open; the board doesn't take input.
    #[default]
    Menu,
    /// @brief A game is being played.
    Playing,
}

/// @brief How strong the AI plays, as a search depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiLevel {
    /// @brief Looks one move ahead: grabs material, misses threats. Good for practice.
    Easy,
    /// @brief Looks two moves ahead.
    Medium,
    /// @brief Looks four moves ahead.
    Hard,
}

impl AiLevel {
    /// @brief Search depth in plies for this level.
    pub fn depth(self) -> u32 {
        match self {
            AiLevel::Easy => 1,
            AiLevel::Medium => 2,
            AiLevel::Hard => 4,
        }
    }
}

/// @brief The choices made in the menu, used when Start is pressed.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchSettings {
    /// @brief Play against the AI (true) or a second human on this computer (false).
    pub vs_ai: bool,
    /// @brief The human's side in an AI game.
    pub human_side: Side,
    /// @brief The AI's strength.
    pub ai_level: AiLevel,
}

impl Default for MatchSettings {
    fn default() -> Self {
        Self {
            vs_ai: true,
            human_side: Side::White,
            ai_level: AiLevel::Medium,
        }
    }
}

/// @brief A clickable menu option.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum MenuButton {
    /// @brief Choose the opponent.
    Opponent {
        /// @brief True for the AI, false for a second human.
        vs_ai: bool,
    },
    /// @brief Choose the human's side in AI games.
    PlayAs(Side),
    /// @brief Choose the AI's strength.
    Level(AiLevel),
    /// @brief Start a game with the current settings.
    Start,
}

/// @brief Marks the full-screen layer holding the menu panel.
#[derive(Component)]
struct MainMenu;

/// @brief Spawns the menu panel: title, option rows, Start button and a controls hint.
fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((MainMenu, centered_layer()))
        .with_children(|layer| {
            layer
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(16.0),
                        padding: UiRect::all(Val::Px(32.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BACKGROUND),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("3D Chess"),
                        TextFont::from_font_size(TITLE_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                        Pickable::IGNORE,
                    ));
                    spawn_option_row(
                        panel,
                        "Opponent",
                        &[
                            (MenuButton::Opponent { vs_ai: true }, "Computer"),
                            (MenuButton::Opponent { vs_ai: false }, "Human"),
                        ],
                    );
                    spawn_option_row(
                        panel,
                        "Play as",
                        &[
                            (MenuButton::PlayAs(Side::White), "White"),
                            (MenuButton::PlayAs(Side::Black), "Black"),
                        ],
                    );
                    spawn_option_row(
                        panel,
                        "AI level",
                        &[
                            (MenuButton::Level(AiLevel::Easy), "Easy"),
                            (MenuButton::Level(AiLevel::Medium), "Medium"),
                            (MenuButton::Level(AiLevel::Hard), "Hard"),
                        ],
                    );
                    spawn_button(panel, MenuButton::Start, "Start", MENU_FONT_SIZE);
                    panel.spawn((
                        Text::new("Right-drag to rotate, scroll to zoom, Esc for this menu"),
                        TextFont::from_font_size(HINT_FONT_SIZE),
                        TextColor(HINT_COLOR),
                        Pickable::IGNORE,
                    ));
                });
        });
}

/// @brief Spawns one labelled row of option buttons.
fn spawn_option_row(panel: &mut ChildSpawnerCommands, label: &str, options: &[(MenuButton, &str)]) {
    panel
        .spawn(Node {
            column_gap: Val::Px(10.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(MENU_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(LABEL_WIDTH),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            for &(button, text) in options {
                spawn_button(row, button, text, MENU_FONT_SIZE);
            }
        });
}

/// @brief Applies menu clicks to the settings; Start sets up the AI, starts a new game and leaves the menu.
fn handle_menu_clicks(
    mut clicks: MessageReader<PointerClick>,
    buttons: Query<&MenuButton>,
    mut settings: ResMut<MatchSettings>,
    mut ai: ResMut<AiPlayer>,
    mut requests: MessageWriter<NewGameRequested>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for click in clicks.read() {
        if click.button != PointerButton::Primary {
            continue;
        }
        let Ok(&button) = buttons.get(click.entity) else {
            continue;
        };
        let mut updated = *settings;
        let start = press(&mut updated, button);
        if updated != *settings {
            debug!(?updated, "menu setting changed");
            *settings = updated;
        }
        if start {
            *ai = ai_player_for(&settings);
            info!(?settings, "match started");
            requests.write(NewGameRequested);
            next_state.set(AppState::Playing);
        }
    }
}

/// @brief Esc during a game opens the menu; the game waits behind it until Start begins a new one.
fn open_menu_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<Selection>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        info!("menu opened");
        *selection = Selection::default();
        next_state.set(AppState::Menu);
    }
}

/// @brief Colors the selected options gold. The Start button is always gold.
fn update_menu_buttons(
    settings: Res<MatchSettings>,
    mut buttons: Query<(&MenuButton, &mut BackgroundColor)>,
) {
    for (&button, mut background) in &mut buttons {
        let wanted = if is_highlighted(&settings, button) {
            SELECTED_BACKGROUND
        } else {
            BUTTON_BACKGROUND
        };
        if background.0 != wanted {
            background.0 = wanted;
        }
    }
}

/// @brief Shows the menu only in the menu state.
fn sync_menu_visibility(
    state: Res<State<AppState>>,
    mut menu: Query<&mut Visibility, With<MainMenu>>,
) {
    set_visible(&mut menu, *state.get() == AppState::Menu);
}

/// @brief Applies one button press to the settings.
/// @return True if it was the Start button.
fn press(settings: &mut MatchSettings, button: MenuButton) -> bool {
    match button {
        MenuButton::Opponent { vs_ai } => settings.vs_ai = vs_ai,
        MenuButton::PlayAs(side) => settings.human_side = side,
        MenuButton::Level(level) => settings.ai_level = level,
        MenuButton::Start => return true,
    }
    false
}

/// @brief Whether a button shows as selected for the current settings.
fn is_highlighted(settings: &MatchSettings, button: MenuButton) -> bool {
    match button {
        MenuButton::Opponent { vs_ai } => settings.vs_ai == vs_ai,
        MenuButton::PlayAs(side) => settings.human_side == side,
        MenuButton::Level(level) => settings.ai_level == level,
        MenuButton::Start => true,
    }
}

/// @brief The AI setup for a match: the side opposite the human, or no AI for two humans.
fn ai_player_for(settings: &MatchSettings) -> AiPlayer {
    AiPlayer {
        side: settings.vs_ai.then(|| settings.human_side.opponent()),
        depth: settings.ai_level.depth(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @brief Buttons change their setting; only Start starts.
    #[test]
    fn buttons_change_settings() {
        let mut settings = MatchSettings::default();
        assert!(!press(&mut settings, MenuButton::PlayAs(Side::Black)));
        assert!(!press(&mut settings, MenuButton::Level(AiLevel::Easy)));
        assert_eq!(settings.human_side, Side::Black);
        assert_eq!(settings.ai_level, AiLevel::Easy);
        assert!(is_highlighted(&settings, MenuButton::Level(AiLevel::Easy)));
        assert!(!is_highlighted(&settings, MenuButton::Level(AiLevel::Hard)));
        assert!(press(&mut settings, MenuButton::Start));
    }

    /// @brief The AI takes the side opposite the human, and sits out human-vs-human games.
    #[test]
    fn ai_setup_follows_settings() {
        let mut settings = MatchSettings {
            vs_ai: true,
            human_side: Side::Black,
            ai_level: AiLevel::Hard,
        };
        let ai = ai_player_for(&settings);
        assert_eq!((ai.side, ai.depth), (Some(Side::White), 4));

        settings.vs_ai = false;
        assert_eq!(ai_player_for(&settings).side, None);
    }
}
