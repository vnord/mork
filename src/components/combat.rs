use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMelee {
    pub light_cooldown: f32,
    pub light_attack_anim_pending: bool,
    pub light_attack_anim_playing: bool,
}

impl Default for PlayerMelee {
    fn default() -> Self {
        Self {
            light_cooldown: 0.0,
            light_attack_anim_pending: false,
            light_attack_anim_playing: false,
        }
    }
}

#[derive(Component)]
pub struct HitFlash {
    pub remaining: f32,
    pub restore: Color,
}
