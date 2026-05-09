use bevy::animation::graph::AnimationNodeIndex;
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct CharacterVisualSetup {
    pub gltf_asset_path: &'static str,
    pub idle_animation_index: usize,
    pub light_attack_animation_index: usize,
    pub hidden_node_names: &'static [&'static str],
    pub weapon_bone_name: Option<&'static str>,
}

#[derive(Component, Clone, Copy)]
pub struct CharacterAnimationNodes {
    pub idle: AnimationNodeIndex,
    pub light_attack: AnimationNodeIndex,
}
