use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

const STICK_DEADZONE: f32 = 0.2;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    Jump,
    Dodge,
    LightAttack,
    HeavyAttack,
    Block,
    LockOn,
}

#[must_use]
pub fn default_input_map() -> InputMap<Action> {
    InputMap::default()
        .with_dual_axis(Action::Move, VirtualDPad::wasd())
        .with_dual_axis(
            Action::Move,
            GamepadStick::LEFT.with_circle_deadzone(STICK_DEADZONE),
        )
        .with(Action::Jump, KeyCode::Space)
        .with(Action::Jump, GamepadButton::South)
        .with(Action::LightAttack, MouseButton::Left)
        .with(Action::LightAttack, GamepadButton::RightTrigger)
        .with(Action::HeavyAttack, MouseButton::Right)
        .with(Action::HeavyAttack, GamepadButton::RightTrigger2)
        .with(Action::Block, KeyCode::ShiftLeft)
        .with(Action::Block, GamepadButton::LeftTrigger)
        .with(Action::Dodge, KeyCode::KeyQ)
        .with(Action::Dodge, GamepadButton::East)
        .with(Action::LockOn, KeyCode::Tab)
        .with(Action::LockOn, GamepadButton::RightThumb)
}
