use bevy::prelude::*;

#[derive(Component)]
pub struct FollowCamera {
    pub look_at_offset: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub mouse_sensitivity: f32,
    pub gamepad_sensitivity: f32,
    pub sharpness: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        let offset = Vec3::new(0.0, 5.0, 10.0);
        let distance = offset.length();

        Self {
            look_at_offset: Vec3::new(0.0, 1.0, 0.0),
            distance,
            yaw: offset.x.atan2(offset.z),
            pitch: (offset.y / distance).asin(),
            min_pitch: -0.35,
            max_pitch: 1.15,
            mouse_sensitivity: 0.004,
            gamepad_sensitivity: 2.5,
            sharpness: 12.0,
        }
    }
}

impl FollowCamera {
    pub fn apply_orbit_delta(&mut self, delta: Vec2) {
        self.yaw -= delta.x;
        self.pitch = (self.pitch + delta.y).clamp(self.min_pitch, self.max_pitch);
    }

    #[must_use]
    pub fn offset(&self) -> Vec3 {
        orbit_offset(self.yaw, self.pitch, self.distance)
    }
}

#[must_use]
pub fn orbit_offset(yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    Vec3::new(
        distance * pitch.cos() * yaw.sin(),
        distance * pitch.sin(),
        distance * pitch.cos() * yaw.cos(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_offset_matches_starting_follow_camera() {
        let offset = FollowCamera::default().offset();

        assert!(offset.x.abs() < 0.001);
        assert!((offset.y - 5.0).abs() < 0.001);
        assert!((offset.z - 10.0).abs() < 0.001);
    }

    #[test]
    fn positive_y_orbit_delta_pitches_camera_up() {
        let mut camera = FollowCamera::default();

        camera.apply_orbit_delta(Vec2::new(0.25, 0.5));

        assert!((camera.yaw + 0.25).abs() < 0.001);
        assert!(camera.pitch > FollowCamera::default().pitch);
    }

    #[test]
    fn pitch_is_clamped() {
        let mut camera = FollowCamera::default();

        camera.apply_orbit_delta(Vec2::new(0.0, 100.0));
        assert!((camera.pitch - camera.max_pitch).abs() < 0.001);

        camera.apply_orbit_delta(Vec2::new(0.0, -100.0));
        assert!((camera.pitch - camera.min_pitch).abs() < 0.001);
    }
}
