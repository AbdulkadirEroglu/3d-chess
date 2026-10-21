//! @file board.rs
//! @brief The 3D board: stone tiles on a base slab, scene lighting, and selection highlights.
//!        Owns the conversion from a `Square` to a world position.

use bevy::picking::Pickable;
use bevy::prelude::*;
use chess_core::Square;

use crate::input::Selection;

/// @brief Width of one square in world units.
pub const SQUARE_SIZE: f32 = 1.0;

/// @brief Thickness of a square tile.
const TILE_HEIGHT: f32 = 0.1;

/// @brief Thickness of the base slab under the tiles.
const BASE_HEIGHT: f32 = 0.35;

/// @brief How far the base slab reaches past the outer squares.
const BASE_MARGIN: f32 = 0.5;

/// @brief Light square color: pale stone.
const LIGHT_SQUARE_COLOR: Color = Color::srgb(0.81, 0.78, 0.71);

/// @brief Dark square color: weathered stone.
const DARK_SQUARE_COLOR: Color = Color::srgb(0.36, 0.34, 0.31);

/// @brief Base slab color: dark stone.
const BASE_COLOR: Color = Color::srgb(0.23, 0.21, 0.25);

/// @brief Sun strength in lux.
const SUN_ILLUMINANCE: f32 = 9_000.0;

/// @brief Soft fill light so shadows aren't pitch black.
const AMBIENT_BRIGHTNESS: f32 = 250.0;

/// @brief Highlight color: translucent gold.
const HIGHLIGHT_COLOR: Color = Color::srgba(0.83, 0.64, 0.30, 0.7);

/// @brief Height of highlight markers above the tiles, so they never flicker into them.
const HIGHLIGHT_LIFT: f32 = 0.01;

/// @brief Spawns the board and lights at startup, and redraws highlights when the selection changes.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_board, spawn_lights, init_highlight_assets))
            .add_systems(
                Update,
                show_highlights.run_if(resource_changed::<Selection>),
            );
    }
}

/// @brief Marks a highlight marker, so the old ones can be removed.
#[derive(Component)]
struct Highlight;

/// @brief Meshes and material for highlights, made once at startup.
#[derive(Resource)]
struct HighlightAssets {
    /// @brief Flat plate on the selected square.
    selected: Handle<Mesh>,
    /// @brief Small dot on an empty target square.
    target: Handle<Mesh>,
    /// @brief Ring around a piece that can be captured.
    capture: Handle<Mesh>,
    /// @brief Shared translucent material.
    material: Handle<StandardMaterial>,
}

/// @brief World position of the center of a square's top surface.
/// @note File a is at -x and rank 1 at +z, so White sits on the camera's near side.
pub fn square_to_world(square: Square) -> Vec3 {
    Vec3::new(
        (f32::from(square.file()) - 3.5) * SQUARE_SIZE,
        0.0,
        (3.5 - f32::from(square.rank())) * SQUARE_SIZE,
    )
}

/// @brief The square under a world position, ignoring height. None when it's off the board.
/// @note Inverse of `square_to_world`; input uses it to turn any clicked point into a square.
pub fn world_to_square(position: Vec3) -> Option<Square> {
    let file = (position.x / SQUARE_SIZE + 4.0).floor();
    let rank = (4.0 - position.z / SQUARE_SIZE).floor();
    if !(0.0..8.0).contains(&file) || !(0.0..8.0).contains(&rank) {
        return None;
    }
    // Both are whole numbers in 0..8 here, so the casts are exact.
    Square::new(file as u8, rank as u8)
}

/// @brief Spawns the 64 square tiles and the base slab under them.
fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stone = |color: Color| StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.85,
        ..default()
    };
    let tile = meshes.add(Cuboid::new(SQUARE_SIZE, TILE_HEIGHT, SQUARE_SIZE));
    let light = materials.add(stone(LIGHT_SQUARE_COLOR));
    let dark = materials.add(stone(DARK_SQUARE_COLOR));

    for square in (0..64).filter_map(Square::from_index) {
        // a1 is dark: a square is dark when file + rank is even.
        let material = if (square.file() + square.rank()) % 2 == 0 {
            dark.clone()
        } else {
            light.clone()
        };
        commands.spawn((
            Name::new(format!("Square {square}")),
            Mesh3d(tile.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(square_to_world(square) - Vec3::Y * TILE_HEIGHT / 2.0),
        ));
    }

    let base_size = 8.0 * SQUARE_SIZE + 2.0 * BASE_MARGIN;
    commands.spawn((
        Name::new("Board base"),
        Mesh3d(meshes.add(Cuboid::new(base_size, BASE_HEIGHT, base_size))),
        MeshMaterial3d(materials.add(stone(BASE_COLOR))),
        Transform::from_xyz(0.0, -TILE_HEIGHT - BASE_HEIGHT / 2.0, 0.0),
    ));
    debug!("board spawned: 64 squares");
}

/// @brief Spawns a sun with shadows and sets a soft ambient fill.
fn spawn_lights(mut commands: Commands, mut ambient: ResMut<GlobalAmbientLight>) {
    ambient.brightness = AMBIENT_BRIGHTNESS;
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: SUN_ILLUMINANCE,
            shadow_maps_enabled: true,
            ..default()
        },
        // From the side and above, so pieces cast visible shadows across the board.
        Transform::from_xyz(5.0, 10.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    debug!("lights spawned");
}

/// @brief Creates the highlight meshes and material.
fn init_highlight_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(HighlightAssets {
        selected: meshes.add(Cuboid::new(SQUARE_SIZE * 0.95, 0.01, SQUARE_SIZE * 0.95)),
        target: meshes.add(Cylinder::new(SQUARE_SIZE * 0.16, 0.01)),
        capture: meshes.add(Torus::new(SQUARE_SIZE * 0.36, SQUARE_SIZE * 0.46)),
        material: materials.add(StandardMaterial {
            base_color: HIGHLIGHT_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    });
}

/// @brief Replaces the highlight markers: one on the selected square, one per legal target.
fn show_highlights(
    mut commands: Commands,
    selection: Res<Selection>,
    assets: Res<HighlightAssets>,
    old: Query<Entity, With<Highlight>>,
) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let Some(selected) = selection.square else {
        return;
    };

    // Promotions list four moves to the same square; mark each target once.
    let mut targets: Vec<_> = selection
        .moves
        .iter()
        .map(|mv| (mv.to, mv.capture.is_some()))
        .collect();
    targets.sort_by_key(|(square, _)| square.index());
    targets.dedup();

    let markers = std::iter::once((selected, &assets.selected)).chain(targets.into_iter().map(
        |(square, capture)| {
            (
                square,
                if capture {
                    &assets.capture
                } else {
                    &assets.target
                },
            )
        },
    ));
    for (square, mesh) in markers {
        commands.spawn((
            Highlight,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(assets.material.clone()),
            Transform::from_translation(square_to_world(square) + Vec3::Y * HIGHLIGHT_LIFT),
            // Markers must not catch clicks meant for the square or piece below.
            Pickable::IGNORE,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @brief Any point within a square maps back to that square; points off the board map to nothing.
    #[test]
    fn world_positions_map_to_squares() {
        for square in (0..64).filter_map(Square::from_index) {
            let center = square_to_world(square);
            for offset in [
                Vec3::ZERO,
                Vec3::new(0.45, 0.8, -0.45),
                Vec3::new(-0.45, 0.0, 0.45),
            ] {
                assert_eq!(world_to_square(center + offset * SQUARE_SIZE), Some(square));
            }
        }
        assert_eq!(world_to_square(Vec3::new(4.2, 0.0, 0.0)), None);
        assert_eq!(world_to_square(Vec3::new(0.0, 0.0, -4.2)), None);
    }
}
