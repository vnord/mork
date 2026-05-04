use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMelee {
    pub light_cooldown: f32,
}

impl Default for PlayerMelee {
    fn default() -> Self {
        Self {
            light_cooldown: 0.0,
        }
    }
}

#[derive(Component)]
pub struct HitFlash {
    pub remaining: f32,
    pub restore: Color,
}
