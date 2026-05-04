use bevy::prelude::*;

use crate::systems::combat::{player_light_attack, tick_hit_flash};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (player_light_attack, tick_hit_flash));
    }
}
