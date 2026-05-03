use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use mork::components::camera::FollowCamera;
use mork::components::transform::PlayerTransform;
use mork::plugins::{combat::CombatPlugin, enemy::EnemyPlugin};
use mork::systems::input::Action;

use bevy_rapier3d::prelude::{Collider, KinematicCharacterController, NoUserData, RigidBody};
use leafwing_input_manager::input_processing::WithDualAxisProcessingPipelineExt;
use leafwing_input_manager::prelude::{
    ActionState, GamepadStick, InputMap, MouseMove, VirtualDPad,
};
use mork::components::player::Player;
use mork::systems::movement::{
    calculate_camera_relative_movement_direction, movement_intent_from_axis,
};

const STICK_DEADZONE: f32 = 0.2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mork".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_rapier3d::prelude::RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(bevy_rapier3d::prelude::RapierDebugRenderPlugin::default())
        .add_plugins(leafwing_input_manager::prelude::InputManagerPlugin::<Action>::default())
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(EnemyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, capture_cursor)
        .add_systems(Update, toggle_cursor_capture)
        .add_systems(Update, move_player)
        .add_systems(Update, follow_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        FollowCamera::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Moonlight"),
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    commands.spawn((
        Name::new("Arena Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(10.0, 0.1, 10.0),
    ));

    let input_map = InputMap::default()
        .with_dual_axis(Action::Move, VirtualDPad::wasd())
        .with_dual_axis(
            Action::Move,
            GamepadStick::LEFT.with_circle_deadzone(STICK_DEADZONE),
        )
        .with_dual_axis(Action::OrbitCameraMouse, MouseMove::default())
        .with_dual_axis(
            Action::OrbitCameraGamepad,
            GamepadStick::RIGHT.with_circle_deadzone(STICK_DEADZONE),
        );

    commands.spawn((
        Name::new("Player"),
        Player,
        PlayerTransform,
        input_map,
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(0.6, 0.4),
        KinematicCharacterController::default(),
    ));
}

fn capture_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}

#[allow(clippy::needless_pass_by_value)]
fn toggle_cursor_capture(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }

    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn move_player(
    time: Res<Time>,
    mut query: Query<(&ActionState<Action>, &mut KinematicCharacterController), With<Player>>,
    camera_transform: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok((action_state, mut controller)) = query.single_mut() else {
        return;
    };

    let Ok(camera_transform) = camera_transform.single() else {
        return;
    };

    let axis = action_state.clamped_axis_pair(&Action::Move);
    let intent = movement_intent_from_axis(axis);
    let direction = calculate_camera_relative_movement_direction(
        &intent,
        camera_transform.rotation * Vec3::NEG_Z,
        camera_transform.rotation * Vec3::X,
    );
    let speed = 5.0;

    controller.translation = Some(direction * speed * time.delta_secs());
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn follow_camera(
    time: Res<Time>,
    player_transform: Query<
        (&Transform, &ActionState<Action>),
        (With<PlayerTransform>, Without<Camera3d>),
    >,
    mut camera_transform: Query<
        (&mut FollowCamera, &mut Transform),
        (With<Camera3d>, Without<PlayerTransform>),
    >,
) {
    let Ok((player_transform, action_state)) = player_transform.single() else {
        return;
    };

    let Ok((mut follow_camera, mut camera_transform)) = camera_transform.single_mut() else {
        return;
    };

    let mouse_delta =
        action_state.axis_pair(&Action::OrbitCameraMouse) * follow_camera.mouse_sensitivity;
    let gamepad_delta = action_state.clamped_axis_pair(&Action::OrbitCameraGamepad)
        * follow_camera.gamepad_sensitivity
        * time.delta_secs();
    let gamepad_delta = Vec2::new(gamepad_delta.x, -gamepad_delta.y);
    follow_camera.apply_orbit_delta(mouse_delta + gamepad_delta);

    let target = player_transform.translation + follow_camera.look_at_offset;
    let desired_position = player_transform.translation + follow_camera.offset();
    let alpha = 1.0 - (-follow_camera.sharpness * time.delta_secs()).exp();

    camera_transform.translation = camera_transform.translation.lerp(desired_position, alpha);
    camera_transform.look_at(target, Vec3::Y);
}
