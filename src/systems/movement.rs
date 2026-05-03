use bevy::prelude::*;

pub struct MovementIntent {
    pub forward: f32,
    pub strafe: f32,
}

#[must_use]
pub const fn movement_intent_from_axis(axis: Vec2) -> MovementIntent {
    MovementIntent {
        forward: axis.y,
        strafe: axis.x,
    }
}

#[must_use]
pub fn calculate_movement_direction(intent: &MovementIntent) -> Vec3 {
    let direction = Vec3::new(intent.strafe, 0.0, -intent.forward);

    direction.normalize_or_zero()
}

#[must_use]
pub fn calculate_camera_relative_movement_direction(
    intent: &MovementIntent,
    camera_forward: Vec3,
    camera_right: Vec3,
) -> Vec3 {
    let forward = flatten(camera_forward);
    let right = flatten(camera_right);

    (forward * intent.forward + right * intent.strafe).normalize_or_zero()
}

#[must_use]
fn flatten(direction: Vec3) -> Vec3 {
    Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;

    fn intent(forward: f32, strafe: f32) -> MovementIntent {
        MovementIntent { forward, strafe }
    }

    #[test]
    fn no_input_returns_zero() {
        assert_eq!(calculate_movement_direction(&intent(0.0, 0.0)), Vec3::ZERO);
    }

    #[test]
    fn forward_is_negative_z() {
        let result = calculate_movement_direction(&intent(1.0, 0.0));
        assert_eq!(result, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn backward_is_positive_z() {
        let result = calculate_movement_direction(&intent(-1.0, 0.0));
        assert_eq!(result, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn left_is_negative_x() {
        let result = calculate_movement_direction(&intent(0.0, -1.0));
        assert_eq!(result, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn right_is_positive_x() {
        let result = calculate_movement_direction(&intent(0.0, 1.0));
        assert_eq!(result, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn forward_right_diagonal_is_normalized() {
        let result = calculate_movement_direction(&intent(1.0, 1.0));
        assert!((result.length() - 1.0).abs() < 0.001);
        assert!(result.x > 0.0);
        assert!(result.z < 0.0);
        assert!(result.y.abs() < 0.001);
    }

    #[test]
    fn opposite_inputs_cancel() {
        let result = calculate_movement_direction(&intent(0.0, 0.0));
        assert_eq!(result, Vec3::ZERO);
    }

    #[test]
    fn y_is_always_zero() {
        let result = calculate_movement_direction(&intent(1.0, -1.0));
        assert!(result.y.abs() < 0.001);
    }

    #[test]
    fn axis_values_map_to_movement_intent() {
        let intent = movement_intent_from_axis(Vec2::new(-0.25, 0.75));
        assert!((intent.forward - 0.75).abs() < 0.001);
        assert!((intent.strafe + 0.25).abs() < 0.001);
    }

    #[test]
    fn camera_relative_forward_uses_flat_camera_forward() {
        let result = calculate_camera_relative_movement_direction(
            &intent(1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::Z,
        );

        assert!((result - Vec3::X).length() < 0.001);
    }

    #[test]
    fn camera_relative_strafe_uses_flat_camera_right() {
        let result = calculate_camera_relative_movement_direction(
            &intent(0.0, -1.0),
            Vec3::NEG_Z,
            Vec3::new(0.0, 1.0, 1.0),
        );

        assert!((result - Vec3::NEG_Z).length() < 0.001);
    }

    #[test]
    fn camera_relative_diagonal_is_normalized() {
        let result =
            calculate_camera_relative_movement_direction(&intent(1.0, 1.0), Vec3::NEG_Z, Vec3::X);

        assert!((result.length() - 1.0).abs() < 0.001);
        assert!(result.x > 0.0);
        assert!(result.z < 0.0);
    }
}
