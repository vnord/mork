use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct CharacterVisualSetup {
    pub gltf_asset_path: &'static str,
    pub idle_animation_index: usize,
    pub hidden_node_names: &'static [&'static str],
}
