use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    OrbitCameraMouse,
    #[actionlike(DualAxis)]
    OrbitCameraGamepad,
    Dodge,
    LightAttack,
    HeavyAttack,
    Block,
    LockOn,
}
