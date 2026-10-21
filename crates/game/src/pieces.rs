//! @file pieces.rs
//! @brief Piece entities: spawned from the active game, one per piece.
//! @note Alpha uses placeholder shapes, taller for more valuable pieces. Theme packs replace them in beta.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use chess_core::{Color as Side, Piece, PieceKind, Square};

use crate::board::square_to_world;
use crate::turn::ActiveGame;

/// @brief White piece color: cream stone.
const WHITE_PIECE_COLOR: Color = Color::srgb(0.93, 0.90, 0.82);

/// @brief Black piece color: dark stone.
const BLACK_PIECE_COLOR: Color = Color::srgb(0.13, 0.12, 0.14);

/// @brief Spawns the pieces once the game exists.
pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pieces);
    }
}

/// @brief Marks a piece entity: which piece it shows and the square it stands on.
#[derive(Component, Debug)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "`piece` is read by the turn flow, alpha step 5")
)]
pub struct PieceOnBoard {
    /// @brief Kind and color.
    pub piece: Piece,
    /// @brief Current square.
    pub square: Square,
}

/// @brief Placeholder mesh for a piece kind, its height, and its rotation.
/// @note The knight's ring is stood upright so it faces the players.
fn placeholder_shape(kind: PieceKind) -> (Mesh, f32, Quat) {
    match kind {
        PieceKind::Pawn => (Mesh::from(Sphere::new(0.2)), 0.4, Quat::IDENTITY),
        PieceKind::Knight => (
            Mesh::from(Torus::new(0.12, 0.28)),
            0.56,
            Quat::from_rotation_x(FRAC_PI_2),
        ),
        PieceKind::Bishop => (Mesh::from(Cone::new(0.25, 0.75)), 0.75, Quat::IDENTITY),
        PieceKind::Rook => (Mesh::from(Cylinder::new(0.28, 0.55)), 0.55, Quat::IDENTITY),
        PieceKind::Queen => (Mesh::from(Capsule3d::new(0.22, 0.5)), 0.94, Quat::IDENTITY),
        PieceKind::King => (
            Mesh::from(Cuboid::new(0.36, 1.05, 0.36)),
            1.05,
            Quat::IDENTITY,
        ),
    }
}

/// @brief Spawns one entity per piece in the active game, standing on its square.
fn spawn_pieces(
    mut commands: Commands,
    game: Res<ActiveGame>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stone = |color: Color| StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.6,
        ..default()
    };
    let white = materials.add(stone(WHITE_PIECE_COLOR));
    let black = materials.add(stone(BLACK_PIECE_COLOR));

    let mut count = 0;
    for (square, piece) in game.0.pieces() {
        let (mesh, height, rotation) = placeholder_shape(piece.kind);
        let material = match piece.color {
            Side::White => white.clone(),
            Side::Black => black.clone(),
        };
        // Shapes are centered on their origin; lift them so they stand on the square.
        let position = square_to_world(square) + Vec3::Y * height / 2.0;
        commands.spawn((
            Name::new(format!("{:?} {:?} {square}", piece.color, piece.kind)),
            PieceOnBoard { piece, square },
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::from_translation(position).with_rotation(rotation),
        ));
        count += 1;
    }
    debug!(count, "pieces spawned");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turn::TurnPlugin;

    /// @brief A new game spawns 32 pieces, each on the square the rules say.
    #[test]
    fn spawns_every_piece_on_its_square() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .add_plugins((TurnPlugin, PiecesPlugin));
        app.update();

        let game = app.world().resource::<ActiveGame>().0.clone();
        let mut query = app.world_mut().query::<(&PieceOnBoard, &Transform)>();
        let spawned: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(spawned.len(), 32);
        for (on_board, transform) in spawned {
            assert_eq!(game.piece_at(on_board.square), Some(on_board.piece));
            let ground = square_to_world(on_board.square);
            assert_eq!(
                (transform.translation.x, transform.translation.z),
                (ground.x, ground.z)
            );
        }
    }
}
