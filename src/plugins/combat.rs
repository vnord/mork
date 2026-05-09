use bevy::prelude::*;

use crate::systems::character_visual::{
    sync_player_light_attack_animation, sync_player_weapon_bone,
};
#[cfg(debug_assertions)]
use crate::systems::combat::debug_draw_light_attack_blade;
use crate::systems::combat::{
    player_light_attack_hit_detection, player_light_attack_input, tick_hit_burst, tick_hit_flash,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_player_weapon_bone,
                player_light_attack_input,
                sync_player_light_attack_animation,
                player_light_attack_hit_detection,
                tick_hit_flash,
                tick_hit_burst,
            )
                .chain(),
        );

        #[cfg(debug_assertions)]
        app.add_systems(PostUpdate, debug_draw_light_attack_blade);
    }
}
