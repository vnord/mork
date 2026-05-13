use bevy::animation::graph::AnimationNodeIndex;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct CharacterVisualSetup {
    pub gltf_handle: Handle<Gltf>,
    pub idle_animation_name: &'static str,
    pub light_attack_animation_name: &'static str,
    pub hidden_node_names: &'static [&'static str],
    pub weapon_bone_name: Option<&'static str>,
}

#[derive(Component, Clone, Copy)]
pub struct CharacterAnimationNodes {
    pub idle: AnimationNodeIndex,
    pub light_attack: AnimationNodeIndex,
}

#[derive(Component)]
pub struct CharacterAnimationInitPending;
