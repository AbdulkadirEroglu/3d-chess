//! @file pieces.rs
//! @brief Piece entities: one per piece in the active game, showing the theme's model or a placeholder shape.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::system::SystemParam;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use chess_core::{Color as Side, Piece, PieceKind, Square};

use crate::board::square_to_world;
use crate::theme::{ThemeReady, ThemeStatus};
use crate::turn::{ActiveGame, GameSet, GameStarted};

/// @brief White placeholder color: cream stone.
const WHITE_PIECE_COLOR: Color = Color::srgb(0.93, 0.90, 0.82);

/// @brief Black placeholder color: dark stone.
const BLACK_PIECE_COLOR: Color = Color::srgb(0.13, 0.12, 0.14);

/// @brief Spawns the pieces whenever a game starts or the theme becomes ready.
pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThemeStatus>()
            .init_resource::<PlaceholderMaterials>()
            .add_message::<ThemeReady>()
            .add_systems(Update, respawn_pieces.in_set(GameSet::Animation));
    }
}

/// @brief Marks a piece entity: which piece it shows and the square it stands on.
#[derive(Component, Debug)]
pub struct PieceOnBoard {
    /// @brief Kind and color.
    pub piece: Piece,
    /// @brief Current square.
    pub square: Square,
}

/// @brief Shared placeholder materials, one per side.
#[derive(Resource)]
struct PlaceholderMaterials {
    /// @brief Cream stone.
    white: Handle<StandardMaterial>,
    /// @brief Dark stone.
    black: Handle<StandardMaterial>,
}

impl FromWorld for PlaceholderMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mut stone = |color: Color| {
            materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.6,
                ..default()
            })
        };
        Self {
            white: stone(WHITE_PIECE_COLOR),
            black: stone(BLACK_PIECE_COLOR),
        }
    }
}

/// @brief Everything needed to spawn a piece: the theme's model if it has one, otherwise a placeholder.
/// @note Used for whole-board respawns and for promotions, so pieces are built in one place.
#[derive(SystemParam)]
pub struct PieceSpawner<'w, 's> {
    /// @brief Spawns and despawns entities.
    pub commands: Commands<'w, 's>,
    /// @brief Which theme is in use.
    theme: Res<'w, ThemeStatus>,
    /// @brief Loads theme models.
    asset_server: Res<'w, AssetServer>,
    /// @brief Stores placeholder meshes.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// @brief Placeholder materials.
    materials: Res<'w, PlaceholderMaterials>,
}

impl PieceSpawner<'_, '_> {
    /// @brief Spawns a piece standing on its square, facing the opponent.
    /// @note Models face +Z, i.e. Blender's Front view (see docs/ASSET_SPEC.md). That's toward White's
    ///       side, so Black's pieces already face their opponent and White's are turned around.
    pub fn spawn(&mut self, piece: Piece, square: Square) -> Entity {
        let facing = match piece.color {
            Side::White => Quat::from_rotation_y(PI),
            Side::Black => Quat::IDENTITY,
        };
        let name = Name::new(format!("{:?} {:?} {square}", piece.color, piece.kind));
        let marker = PieceOnBoard { piece, square };
        let ground = square_to_world(square);

        if let Some(path) = self.theme.model_path(piece) {
            let model = self
                .asset_server
                .load(GltfAssetLabel::Scene(0).from_asset(path));
            return self
                .commands
                .spawn((
                    name,
                    marker,
                    WorldAssetRoot(model),
                    Transform::from_translation(ground).with_rotation(facing),
                ))
                .id();
        }

        let (mesh, height, rotation) = placeholder_shape(piece.kind);
        let material = match piece.color {
            Side::White => self.materials.white.clone(),
            Side::Black => self.materials.black.clone(),
        };
        // Placeholder shapes are centered on their origin; lift them so they stand on the square.
        self.commands
            .spawn((
                name,
                marker,
                Mesh3d(self.meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(ground + Vec3::Y * height / 2.0)
                    .with_rotation(facing * rotation),
            ))
            .id()
    }
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

/// @brief When a game starts or the theme becomes ready, replaces all piece entities from the active game.
fn respawn_pieces(
    mut started: MessageReader<GameStarted>,
    mut theme_ready: MessageReader<ThemeReady>,
    game: Res<ActiveGame>,
    old: Query<Entity, With<PieceOnBoard>>,
    mut spawner: PieceSpawner,
) {
    // Both readers must be drained every frame, so count them separately.
    let started = started.read().count();
    let theme_ready = theme_ready.read().count();
    if started + theme_ready == 0 {
        return;
    }
    for entity in &old {
        spawner.commands.entity(entity).despawn();
    }
    for (square, piece) in game.0.pieces() {
        spawner.spawn(piece, square);
    }
    debug!(count = game.0.pieces().count(), "pieces spawned");
}

#[cfg(test)]
mod tests {
    use chess_core::Game;

    use super::*;
    use crate::test_support::headless_app;

    /// @brief A new game spawns 32 pieces, each on the square the rules say; a restart doesn't duplicate them.
    #[test]
    fn spawns_every_piece_on_its_square() {
        let mut app = headless_app(Game::new());

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

        app.world_mut().write_message(GameStarted);
        app.update();
        let mut query = app.world_mut().query::<&PieceOnBoard>();
        assert_eq!(query.iter(app.world()).count(), 32);
    }
}
