//! @file animation.rs
//! @brief The animation queue: plays applied moves as visual steps (slide, capture, promotion), one at a time.
//! @note Visual only. The rules have already changed when a step plays; steps never touch game state.

use std::collections::VecDeque;
use std::f32::consts::PI;

use bevy::prelude::*;
use chess_core::{Piece, PieceKind, Square};

use crate::board::square_to_world;
use crate::pieces::{PieceOnBoard, PieceSpawner};
use crate::turn::{GameSet, GameStarted, MovePlayed};

/// @brief Seconds for a piece to slide to its square.
const SLIDE_SECONDS: f32 = 0.45;

/// @brief Seconds for a captured piece to shrink away.
const CAPTURE_SECONDS: f32 = 0.35;

/// @brief Peak height of a knight's hop.
const KNIGHT_JUMP_HEIGHT: f32 = 0.8;

/// @brief How far a captured piece sinks while it shrinks.
const CAPTURE_SINK: f32 = 0.4;

/// @brief Queues steps for every played move and plays them.
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationQueue>().add_systems(
            Update,
            (reset_on_new_game, queue_moves, play_animations)
                .chain()
                .in_set(GameSet::Animation),
        );
    }
}

/// @brief Steps waiting to play; the front one is playing.
#[derive(Resource, Default)]
pub struct AnimationQueue {
    /// @brief Pending steps in order.
    steps: VecDeque<Step>,
    /// @brief Seconds spent on the front step.
    elapsed: f32,
}

impl AnimationQueue {
    /// @brief True while anything is still animating. Input waits until it's false.
    pub fn is_busy(&self) -> bool {
        !self.steps.is_empty()
    }
}

/// @brief One visual step.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// @brief Move a piece between two points; knights hop.
    Slide {
        /// @brief The piece entity.
        entity: Entity,
        /// @brief Start position.
        from: Vec3,
        /// @brief End position.
        to: Vec3,
        /// @brief Arc through the air instead of gliding.
        jump: bool,
    },
    /// @brief Shrink and sink a captured piece, then remove it.
    Capture {
        /// @brief The captured piece entity.
        entity: Entity,
        /// @brief Where it stands.
        from: Vec3,
    },
    /// @brief Replace a promoted pawn with its new piece.
    Promote {
        /// @brief The pawn entity.
        entity: Entity,
        /// @brief The piece it becomes.
        piece: Piece,
        /// @brief Where it stands.
        square: Square,
    },
}

/// @brief Drops pending animations when a new game starts; the pieces get respawned from scratch.
fn reset_on_new_game(
    mut commands: Commands,
    mut started: MessageReader<GameStarted>,
    mut queue: ResMut<AnimationQueue>,
) {
    if started.read().count() == 0 || !queue.is_busy() {
        return;
    }
    // A piece mid-capture has no `PieceOnBoard`, so the respawn wouldn't remove it.
    for step in queue.steps.drain(..) {
        if let Step::Capture { entity, .. } = step
            && let Ok(mut captured) = commands.get_entity(entity)
        {
            captured.despawn();
        }
    }
    queue.elapsed = 0.0;
    debug!("animations cleared for new game");
}

/// @brief Turns each played move into steps and updates the piece components to match the rules.
fn queue_moves(
    mut commands: Commands,
    mut played: MessageReader<MovePlayed>,
    mut queue: ResMut<AnimationQueue>,
    mut pieces: Query<(Entity, &mut PieceOnBoard, &Transform)>,
) {
    for MovePlayed(mv) in played.read() {
        // Entity on a square and its resting height above the board.
        let find = |square| {
            pieces
                .iter()
                .find(|(_, on_board, _)| on_board.square == square)
                .map(|(entity, _, transform)| (entity, transform.translation.y))
        };
        let Some((mover, mover_y)) = find(mv.from) else {
            warn!(?mv, "no piece entity on the from square");
            continue;
        };
        let victim = mv
            .capture
            .and_then(|(_, square)| Some((find(square)?, square)));
        let rook = mv
            .rook_move
            .and_then(|(from, to)| Some((find(from)?, from, to)));

        queue.steps.push_back(Step::Slide {
            entity: mover,
            from: square_to_world(mv.from) + Vec3::Y * mover_y,
            to: square_to_world(mv.to) + Vec3::Y * mover_y,
            jump: mv.piece.kind == PieceKind::Knight,
        });
        if let Some(((entity, y), square)) = victim {
            // No longer on the board; the entity lives on only until its capture step ends.
            commands.entity(entity).remove::<PieceOnBoard>();
            queue.steps.push_back(Step::Capture {
                entity,
                from: square_to_world(square) + Vec3::Y * y,
            });
        }
        if let Some(((entity, y), from, to)) = rook {
            queue.steps.push_back(Step::Slide {
                entity,
                from: square_to_world(from) + Vec3::Y * y,
                to: square_to_world(to) + Vec3::Y * y,
                jump: false,
            });
            if let Ok((_, mut on_board, _)) = pieces.get_mut(entity) {
                on_board.square = to;
            }
        }
        if let Some(kind) = mv.promotion {
            queue.steps.push_back(Step::Promote {
                entity: mover,
                piece: Piece {
                    kind,
                    color: mv.piece.color,
                },
                square: mv.to,
            });
        }

        // Components follow the rules right away; the steps only catch the visuals up.
        if let Ok((_, mut on_board, _)) = pieces.get_mut(mover) {
            on_board.square = mv.to;
            if let Some(kind) = mv.promotion {
                on_board.piece.kind = kind;
            }
        }
    }
}

/// @brief Advances the front step; moves on to the next one when it finishes.
fn play_animations(
    time: Res<Time>,
    mut queue: ResMut<AnimationQueue>,
    mut transforms: Query<&mut Transform>,
    mut spawner: PieceSpawner,
) {
    let Some(&step) = queue.steps.front() else {
        return;
    };
    if queue.elapsed == 0.0 {
        debug!(?step, "animation step started");
    }
    queue.elapsed += time.delta_secs();
    let duration = match step {
        Step::Slide { .. } => SLIDE_SECONDS,
        Step::Capture { .. } => CAPTURE_SECONDS,
        Step::Promote { .. } => 0.0,
    };
    let t = if duration > 0.0 {
        (queue.elapsed / duration).min(1.0)
    } else {
        1.0
    };
    // Smoothstep: slow start, slow finish.
    let eased = t * t * (3.0 - 2.0 * t);

    match step {
        Step::Slide {
            entity,
            from,
            to,
            jump,
        } => {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                let hop = if jump {
                    (PI * t).sin() * KNIGHT_JUMP_HEIGHT
                } else {
                    0.0
                };
                transform.translation = from.lerp(to, eased) + Vec3::Y * hop;
            }
        }
        Step::Capture { entity, from } => {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.scale = Vec3::splat(1.0 - eased);
                transform.translation = from - Vec3::Y * CAPTURE_SINK * eased;
            }
            if t >= 1.0 {
                spawner.commands.entity(entity).despawn();
            }
        }
        Step::Promote {
            entity,
            piece,
            square,
        } => {
            spawner.commands.entity(entity).despawn();
            spawner.spawn(piece, square);
        }
    }

    if t >= 1.0 {
        queue.steps.pop_front();
        queue.elapsed = 0.0;
        if queue.steps.is_empty() {
            debug!("animations finished");
        }
    }
}

#[cfg(test)]
mod tests {
    use chess_core::Game;

    use super::*;
    use crate::test_support::{headless_app, play};
    use crate::turn::ActiveGame;

    /// @brief Every piece entity shows the rules' piece, rests on its square, and nothing extra is left.
    fn assert_matches_rules(app: &mut App) {
        let game = app.world().resource::<ActiveGame>().0.clone();
        let mut query = app.world_mut().query::<(&PieceOnBoard, &Transform)>();
        let pieces: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pieces.len(), game.pieces().count());
        for (on_board, transform) in pieces {
            assert_eq!(game.piece_at(on_board.square), Some(on_board.piece));
            let ground = square_to_world(on_board.square);
            assert!((transform.translation.x - ground.x).abs() < 1e-4);
            assert!((transform.translation.z - ground.z).abs() < 1e-4);
            assert_eq!(transform.scale, Vec3::ONE);
        }
    }

    /// @brief After a capture the captured entity is gone and the rest match the rules.
    #[test]
    fn capture() {
        let mut app = headless_app(Game::new());
        for mv in ["e2e4", "d7d5", "e4d5", "g8f6"] {
            play(&mut app, mv);
        }
        assert_eq!(app.world().resource::<ActiveGame>().0.pieces().count(), 31);
        assert_matches_rules(&mut app);
    }

    /// @brief Castling moves the rook entity too; en passant removes the pawn beside the target.
    #[test]
    fn castling_and_en_passant() {
        let game = Game::from_fen("r3k2r/pppppppp/8/4P3/8/8/8/R3K2R b KQkq - 0 1").expect("fen");
        let mut app = headless_app(game);
        for mv in ["d7d5", "e5d6", "e8c8", "e1g1"] {
            play(&mut app, mv);
        }
        assert_matches_rules(&mut app);
    }

    /// @brief A promoted pawn entity becomes a queen.
    #[test]
    fn promotion() {
        let game = Game::from_fen("8/P7/8/8/8/8/8/k6K w - - 0 1").expect("fen");
        let mut app = headless_app(game);
        play(&mut app, "a7a8q");
        assert_matches_rules(&mut app);
    }
}
