//! @file clips.rs
//! @brief Model animation clips: hooks a piece model's idle/walk/attack/hit/die clips to its animation
//!        player, and plays them with a short crossfade.
//! @note Pieces without clips (placeholders, static models) simply have no `PieceAnimator`.

use std::collections::HashMap;
use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::gltf::Gltf;
use bevy::prelude::*;

use crate::pieces::PieceModel;

/// @brief Blend time when switching clips, so motions don't snap.
const CROSSFADE: Duration = Duration::from_millis(150);

/// @brief Hooks up animation clips as models finish loading and starts their idle loop.
pub struct ClipsPlugin;

impl Plugin for ClipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (attach_animators, start_idle).chain());
    }
}

/// @brief The standard clips every piece model provides (docs/ASSET_SPEC.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    /// @brief Resting loop.
    Idle,
    /// @brief Moving loop.
    Walk,
    /// @brief Striking forward.
    Attack,
    /// @brief Taking a blow.
    Hit,
    /// @brief Falling; ends on the ground.
    Die,
}

impl Clip {
    /// @brief All clips, in index order.
    const ALL: [Clip; 5] = [Clip::Idle, Clip::Walk, Clip::Attack, Clip::Hit, Clip::Die];

    /// @brief Clip name inside the model file.
    fn name(self) -> &'static str {
        match self {
            Clip::Idle => "idle",
            Clip::Walk => "walk",
            Clip::Attack => "attack",
            Clip::Hit => "hit",
            Clip::Die => "die",
        }
    }

    /// @brief Whether the clip repeats until something else plays.
    fn loops(self) -> bool {
        matches!(self, Clip::Idle | Clip::Walk)
    }
}

/// @brief A piece model's animation hookup: the entity with its player, and a graph node per clip.
#[derive(Component, Debug)]
pub struct PieceAnimator {
    /// @brief Descendant entity holding the `AnimationPlayer`.
    player: Entity,
    /// @brief Graph node for each clip in `Clip::ALL` order; None if the model lacks it.
    nodes: [Option<AnimationNodeIndex>; 5],
}

/// @brief Animation graph and clip nodes built once per model file, shared by its pieces.
#[derive(Clone)]
struct ModelClips {
    /// @brief Graph holding the model's clips.
    graph: Handle<AnimationGraph>,
    /// @brief Graph node per clip.
    nodes: [Option<AnimationNodeIndex>; 5],
}

/// @brief Gives each loaded piece model a `PieceAnimator` once its scene and clips are available.
fn attach_animators(
    mut commands: Commands,
    models: Query<(Entity, &PieceModel), Without<PieceAnimator>>,
    children: Query<&Children>,
    players: Query<(), With<AnimationPlayer>>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut cache: Local<HashMap<AssetId<Gltf>, ModelClips>>,
) {
    for (root, model) in &models {
        let Some(gltf) = gltfs.get(&model.0) else {
            continue;
        };
        // A model without clips never gets a player; mark it so it isn't searched again.
        if gltf.named_animations.is_empty() {
            commands.entity(root).insert(PieceAnimator {
                player: root,
                nodes: [None; 5],
            });
            continue;
        }
        // The player appears once the model's scene has spawned under the piece.
        let Some(player) = children
            .iter_descendants(root)
            .find(|&entity| players.contains(entity))
        else {
            continue;
        };
        let clips = cache
            .entry(model.0.id())
            .or_insert_with(|| {
                let found: Vec<_> = Clip::ALL
                    .iter()
                    .enumerate()
                    .filter_map(|(index, clip)| {
                        Some((index, gltf.named_animations.get(clip.name())?.clone()))
                    })
                    .collect();
                let (graph, graph_nodes) =
                    AnimationGraph::from_clips(found.iter().map(|(_, clip)| clip.clone()));
                let mut nodes = [None; 5];
                for ((index, _), node) in found.iter().zip(graph_nodes) {
                    nodes[*index] = Some(node);
                }
                debug!(clips = found.len(), "animation graph built for a model");
                ModelClips {
                    graph: graphs.add(graph),
                    nodes,
                }
            })
            .clone();
        commands.entity(player).insert((
            AnimationGraphHandle(clips.graph),
            AnimationTransitions::new(),
        ));
        commands.entity(root).insert(PieceAnimator {
            player,
            nodes: clips.nodes,
        });
    }
}

/// @brief Starts the idle loop on newly animated pieces.
fn start_idle(added: Query<Entity, Added<PieceAnimator>>, mut clips: ClipPlayer) {
    for entity in &added {
        clips.play(entity, Clip::Idle);
    }
}

/// @brief Plays clips on pieces by entity.
#[derive(SystemParam)]
pub struct ClipPlayer<'w, 's> {
    /// @brief Each animated piece's hookup.
    animators: Query<'w, 's, &'static PieceAnimator>,
    /// @brief The players and their transition state.
    players: Query<
        'w,
        's,
        (
            &'static mut AnimationPlayer,
            &'static mut AnimationTransitions,
        ),
    >,
}

impl ClipPlayer<'_, '_> {
    /// @brief True if the piece's model has this clip.
    pub fn has(&self, entity: Entity, clip: Clip) -> bool {
        self.animators
            .get(entity)
            .is_ok_and(|animator| animator.nodes[clip as usize].is_some())
    }

    /// @brief Crossfades a piece to a clip; looping clips repeat.
    /// @return False if the piece has no such clip (e.g. a placeholder).
    pub fn play(&mut self, entity: Entity, clip: Clip) -> bool {
        let Ok(animator) = self.animators.get(entity) else {
            return false;
        };
        let Some(node) = animator.nodes[clip as usize] else {
            return false;
        };
        let Ok((mut player, mut transitions)) = self.players.get_mut(animator.player) else {
            return false;
        };
        let active = transitions.play(&mut player, node, CROSSFADE);
        if clip.loops() {
            active.repeat();
        }
        true
    }
}
