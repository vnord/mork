use bevy::prelude::*;

use crate::systems::character_visual::finish_character_visual_animations;

pub struct CharacterVisualPlugin;

impl Plugin for CharacterVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, finish_character_visual_animations);
    }
}
