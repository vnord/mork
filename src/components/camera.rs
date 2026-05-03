use bevy::prelude::*;

#[derive(Component)]
pub struct FollowCamera {
    pub offset: Vec3,
    pub look_at_offset: Vec3,
    pub sharpness: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 5.0, 10.0),
            look_at_offset: Vec3::new(0.0, 1.0, 0.0),
            sharpness: 12.0,
        }
    }
}
