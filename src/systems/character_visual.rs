use crate::components::character_visual::{CharacterAnimationNodes, CharacterVisualSetup};
use crate::components::combat::PlayerMelee;
use crate::components::player::Player;
use bevy::animation::RepeatAnimation;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

pub const KAYKIT_IDLE_ANIMATION_INDEX: usize = 36;
pub const KAYKIT_LIGHT_ATTACK_ANIMATION_INDEX: usize = 2;

pub const KNIGHT_HIDDEN_NODES: &[&str] = &[
    "1H_Sword_Offhand",
    "Badge_Shield",
    "Rectangle_Shield",
    "Round_Shield",
    "Spike_Shield",
    "2H_Sword",
];

pub const ROGUE_HIDDEN_NODES: &[&str] =
    &["Knife_Offhand", "1H_Crossbow", "2H_Crossbow", "Throwable"];

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::manual_contains
)]
pub fn character_visual_scene_ready(
    trigger: On<SceneInstanceReady>,
    setups: Query<&CharacterVisualSetup>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
    children: Query<&Children>,
    names: Query<&Name>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let root = trigger.entity;
    let Ok(setup) = setups.get(root) else {
        return;
    };

    for descendant in children.iter_descendants(root) {
        let Ok(name) = names.get(descendant) else {
            continue;
        };
        let label = name.as_str();
        if setup
            .hidden_node_names
            .iter()
            .any(|hidden| *hidden == label)
        {
            commands.entity(descendant).insert(Visibility::Hidden);
        }
    }

    let idle_clip = asset_server.load(
        GltfAssetLabel::Animation(setup.idle_animation_index).from_asset(setup.gltf_asset_path),
    );
    let light_clip = asset_server.load(
        GltfAssetLabel::Animation(setup.light_attack_animation_index)
            .from_asset(setup.gltf_asset_path),
    );
    let (graph, node_indices) = AnimationGraph::from_clips([idle_clip, light_clip]);
    let graph_handle = graphs.add(graph);
    let idle_node = node_indices[0];
    let light_node = node_indices[1];
    commands.entity(root).insert(CharacterAnimationNodes {
        idle: idle_node,
        light_attack: light_node,
    });

    for descendant in children.iter_descendants(root) {
        let Ok(mut player) = players.get_mut(descendant) else {
            continue;
        };
        player.play(idle_node).repeat();
        commands
            .entity(descendant)
            .insert(AnimationGraphHandle(graph_handle.clone()));
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub fn sync_player_light_attack_animation(
    mut players: Query<(Entity, &mut PlayerMelee), With<Player>>,
    children: Query<&Children>,
    anim_nodes: Query<&CharacterAnimationNodes>,
    mut anim_players: Query<&mut AnimationPlayer>,
) {
    for (player_entity, mut melee) in &mut players {
        let Ok(kids) = children.get(player_entity) else {
            continue;
        };
        let visual_root = kids.iter().find(|e| anim_nodes.contains(*e));
        let Some(visual_root) = visual_root else {
            continue;
        };
        let Ok(nodes) = anim_nodes.get(visual_root) else {
            continue;
        };

        if melee.light_attack_anim_pending {
            for entity in children.iter_descendants(visual_root) {
                let Ok(mut player) = anim_players.get_mut(entity) else {
                    continue;
                };
                player.stop(nodes.idle);
                player
                    .start(nodes.light_attack)
                    .set_repeat(RepeatAnimation::Never);
            }
            melee.light_attack_anim_pending = false;
            melee.light_attack_anim_playing = true;
        }

        if !melee.light_attack_anim_playing {
            continue;
        }

        let mut saw_player = false;
        let mut all_finished = true;
        for entity in children.iter_descendants(visual_root) {
            let Ok(player) = anim_players.get(entity) else {
                continue;
            };
            saw_player = true;
            if !player.all_finished() {
                all_finished = false;
                break;
            }
        }

        if !(saw_player && all_finished) {
            continue;
        }

        for entity in children.iter_descendants(visual_root) {
            let Ok(mut player) = anim_players.get_mut(entity) else {
                continue;
            };
            player.stop(nodes.light_attack);
            player.start(nodes.idle).repeat();
        }
        melee.light_attack_anim_playing = false;
    }
}
