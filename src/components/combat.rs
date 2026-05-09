use bevy::color::LinearRgba;
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMelee {
    pub light_cooldown: f32,
    pub light_attack_anim_pending: bool,
    pub light_attack_anim_playing: bool,
    pub light_attack_started_secs: Option<f32>,
    pub light_attack_hit_registered: bool,
}

impl Default for PlayerMelee {
    fn default() -> Self {
        Self {
            light_cooldown: 0.0,
            light_attack_anim_pending: false,
            light_attack_anim_playing: false,
            light_attack_started_secs: None,
            light_attack_hit_registered: false,
        }
    }
}

#[derive(Component)]
pub struct HitFlash {
    pub remaining: f32,
    pub restore_base_color: Color,
    pub restore_emissive: LinearRgba,
}

#[derive(Component)]
pub struct HitBurst {
    pub ttl: f32,
}

#[derive(Component)]
pub struct PlayerWeaponBone(pub Entity);
