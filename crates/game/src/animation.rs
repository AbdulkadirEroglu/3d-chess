//! @file animation.rs
//! @brief The animation queue: plays applied moves as visual steps, one at a time.
//!        A capture plays as: walk up to the victim → attack → victim hit → victim dies → walk onto the square.
//! @note Visual only. The rules have already changed when a step plays; steps never touch game state.
//!       Pieces without clips (placeholders) skip the clip steps and shrink away when captured.

use std::collections::VecDeque;
use std::f32::consts::PI;

use bevy::prelude::*;
use chess_core::{Piece, PieceKind, Square};

use crate::board::{SQUARE_SIZE, square_to_world};
use crate::clips::{Clip, ClipPlayer};
use crate::pieces::{PieceOnBoard, PieceSpawner, army_facing};
use crate::turn::{GameSet, GameStarted, MovePlayed};

/// @brief Walking speed in squares per second.
const WALK_SPEED: f32 = 2.5;

/// @brief Shortest walk, so one-square moves don't look rushed.
const MIN_WALK_SECONDS: f32 = 0.3;

/// @brief How far short of its victim an attacker stops, in squares.
const ATTACK_GAP: f32 = 0.6;

/// @brief Seconds for the attack clip.
const ATTACK_SECONDS: f32 = 0.6;

/// @brief Seconds for the hit clip.
const HIT_SECONDS: f32 = 0.35;

/// @brief Seconds for the die clip before the piece is removed.
const DIE_SECONDS: f32 = 0.9;

/// @brief Seconds for a placeholder (no clips) to shrink away when captured.
const SHRINK_SECONDS: f32 = 0.35;

/// @brief How far a shrinking placeholder sinks.
const SHRINK_SINK: f32 = 0.4;

/// @brief Peak height of a knight's hop.
const KNIGHT_JUMP_HEIGHT: f32 = 0.8;

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
    /// @brief Walk a piece between two points, facing the way it goes; knights hop.
    Walk {
        /// @brief The piece entity.
        entity: Entity,
        /// @brief Start position.
        from: Vec3,
        /// @brief End position.
        to: Vec3,
        /// @brief Arc through the air instead of walking.
        jump: bool,
        /// @brief How long the walk takes.
        seconds: f32,
        /// @brief Facing to settle into at the end (back toward the opponent); None keeps facing the path.
        settle: Option<Quat>,
    },
    /// @brief Play a one-shot clip (attack, hit) on a piece; skipped if the piece has no such clip.
    Act {
        /// @brief The piece entity.
        entity: Entity,
        /// @brief The clip.
        clip: Clip,
        /// @brief How long to wait for it.
        seconds: f32,
    },
    /// @brief A captured piece dies (or shrinks, without clips), then is removed.
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

impl Step {
    /// @brief How long the step plays. Clip steps take no time on pieces without that clip.
    fn duration(self, clips: &ClipPlayer) -> f32 {
        match self {
            Step::Walk { seconds, .. } => seconds,
            Step::Act {
                entity,
                clip,
                seconds,
            } if clips.has(entity, clip) => seconds,
            Step::Act { .. } | Step::Promote { .. } => 0.0,
            Step::Capture { entity, .. } if clips.has(entity, Clip::Die) => DIE_SECONDS,
            Step::Capture { .. } => SHRINK_SECONDS,
        }
    }
}

/// @brief A walk step between two points, timed by distance.
fn walk(entity: Entity, from: Vec3, to: Vec3, jump: bool, settle: Option<Quat>) -> Step {
    let squares = from.distance(to) / SQUARE_SIZE;
    Step::Walk {
        entity,
        from,
        to,
        jump,
        seconds: (squares / WALK_SPEED).max(MIN_WALK_SECONDS),
        settle,
    }
}

/// @brief Where an attacker stops: just short of its victim, on the line from its start.
fn approach_point(from: Vec3, victim: Vec3) -> Vec3 {
    let gap = ATTACK_GAP * SQUARE_SIZE;
    let direction = Vec3::new(victim.x - from.x, 0.0, victim.z - from.z);
    let stop = victim - direction.normalize_or_zero() * gap;
    Vec3::new(stop.x, from.y, stop.z)
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

        let lift = Vec3::Y * mover_y;
        let start = square_to_world(mv.from) + lift;
        let end = square_to_world(mv.to) + lift;
        let facing = Some(army_facing(mv.piece.color));
        let jump = mv.piece.kind == PieceKind::Knight;

        if let Some(((victim, victim_y), victim_square)) = victim {
            let victim_at = square_to_world(victim_square) + Vec3::Y * victim_y;
            let approach = approach_point(start, victim_at);
            // No longer on the board; the entity lives on only until its capture step ends.
            commands.entity(victim).remove::<PieceOnBoard>();
            queue.steps.extend([
                walk(mover, start, approach, jump, None),
                Step::Act {
                    entity: mover,
                    clip: Clip::Attack,
                    seconds: ATTACK_SECONDS,
                },
                Step::Act {
                    entity: victim,
                    clip: Clip::Hit,
                    seconds: HIT_SECONDS,
                },
                Step::Capture {
                    entity: victim,
                    from: victim_at,
                },
                walk(mover, approach, end, false, facing),
            ]);
        } else {
            queue.steps.push_back(walk(mover, start, end, jump, facing));
        }

        if let Some(((entity, y), from, to)) = rook {
            let lift = Vec3::Y * y;
            queue.steps.push_back(walk(
                entity,
                square_to_world(from) + lift,
                square_to_world(to) + lift,
                false,
                facing,
            ));
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

/// @brief Advances the front step: starts its clip, moves it along, and settles it when done.
fn play_animations(
    time: Res<Time>,
    mut queue: ResMut<AnimationQueue>,
    mut transforms: Query<&mut Transform>,
    mut clips: ClipPlayer,
    mut spawner: PieceSpawner,
) {
    let Some(&step) = queue.steps.front() else {
        return;
    };
    let starting = queue.elapsed == 0.0;
    if starting {
        debug!(?step, "animation step started");
    }
    queue.elapsed += time.delta_secs();

    let duration = step.duration(&clips);
    let t = if duration > 0.0 {
        (queue.elapsed / duration).min(1.0)
    } else {
        1.0
    };
    // Smoothstep: slow start, slow finish.
    let eased = t * t * (3.0 - 2.0 * t);

    match step {
        Step::Walk {
            entity,
            from,
            to,
            jump,
            settle,
            ..
        } => {
            if starting {
                clips.play(entity, Clip::Walk);
            }
            if let Ok(mut transform) = transforms.get_mut(entity) {
                let hop = if jump {
                    (PI * t).sin() * KNIGHT_JUMP_HEIGHT
                } else {
                    0.0
                };
                transform.translation = from.lerp(to, eased) + Vec3::Y * hop;
                let heading = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
                // Face the path while walking (models face +Z); turn back to the opponent when settling.
                if t < 1.0 && heading.length_squared() > 0.0 {
                    transform.rotation = Quat::from_rotation_y(heading.x.atan2(heading.z));
                } else if t >= 1.0
                    && let Some(facing) = settle
                {
                    transform.rotation = facing;
                }
            }
            if t >= 1.0 {
                clips.play(entity, Clip::Idle);
            }
        }
        Step::Act { entity, clip, .. } => {
            if starting {
                clips.play(entity, clip);
            }
            if t >= 1.0 && clip != Clip::Hit {
                clips.play(entity, Clip::Idle);
            }
        }
        Step::Capture { entity, from } => {
            if starting && clips.play(entity, Clip::Die) {
                debug!("die clip playing");
            }
            if !clips.has(entity, Clip::Die)
                && let Ok(mut transform) = transforms.get_mut(entity)
            {
                transform.scale = Vec3::splat(1.0 - eased);
                transform.translation = from - Vec3::Y * SHRINK_SINK * eased;
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

    /// @brief Every piece entity shows the rules' piece, rests on its square facing the opponent,
    ///        and nothing extra is left.
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
            assert!(
                transform
                    .rotation
                    .angle_between(army_facing(on_board.piece.color))
                    < 1e-3
            );
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

    /// @brief An attacker stops short of its victim, on the line toward it, at its own height.
    #[test]
    fn attacker_stops_short() {
        let stop = approach_point(Vec3::new(0.0, 0.2, 0.0), Vec3::new(2.0, 0.5, 0.0));
        assert!((stop - Vec3::new(2.0 - ATTACK_GAP * SQUARE_SIZE, 0.2, 0.0)).length() < 1e-5);
    }
}
