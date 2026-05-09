use crate::components::character_visual::CharacterVisualSetup;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

pub const KAYKIT_IDLE_ANIMATION_INDEX: usize = 36;

pub const KNIGHT_HIDDEN_NODES: &[&str] = &[
    "1H_Sword_Offhand",
    "Badge_Shield",
    "Rectangle_Shield",
    "Round_Shield",
    "Spike_Shield",
    "2H_Sword",
];

pub const ROGUE_HIDDEN_NODES: &[&str] = &[
    "Knife_Offhand",
    "1H_Crossbow",
    "2H_Crossbow",
    "Throwable",
];

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

    let (graph, animation_index) = AnimationGraph::from_clip(asset_server.load(
        GltfAssetLabel::Animation(setup.idle_animation_index).from_asset(setup.gltf_asset_path),
    ));
    let graph_handle = graphs.add(graph);

    for descendant in children.iter_descendants(root) {
        let Ok(mut player) = players.get_mut(descendant) else {
            continue;
        };
        player.play(animation_index).repeat();
        commands
            .entity(descendant)
            .insert(AnimationGraphHandle(graph_handle.clone()));
    }
}
