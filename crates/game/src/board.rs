//! @file board.rs
//! @brief The 3D board: stone tiles on a base slab, plus the scene lighting.
//!        Owns the conversion from a `Square` to a world position.

use bevy::prelude::*;
use chess_core::Square;

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

/// @brief Spawns the board and lights at startup.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_board, spawn_lights));
    }
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
