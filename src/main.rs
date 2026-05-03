use bevy::prelude::*;
use mork::components::transform::PlayerTransform;
use mork::plugins::{combat::CombatPlugin, enemy::EnemyPlugin};
use mork::systems::input::Action;

use bevy_rapier3d::prelude::{Collider, KinematicCharacterController, NoUserData, RigidBody};
use leafwing_input_manager::prelude::{ActionState, GamepadStick, InputMap, VirtualDPad};
use mork::components::player::Player;
use mork::systems::movement::{calculate_movement_direction, movement_intent_from_axis};

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
        .with_dual_axis(Action::Move, GamepadStick::LEFT);

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

fn move_player(
    time: Res<Time>,
    mut query: Query<(&ActionState<Action>, &mut KinematicCharacterController), With<Player>>,
) {
    let Ok((action_state, mut controller)) = query.single_mut() else {
        return;
    };

    let axis = action_state.clamped_axis_pair(&Action::Move);
    let intent = movement_intent_from_axis(axis);
    let direction = calculate_movement_direction(&intent);
    let speed = 5.0;

    controller.translation = Some(direction * speed * time.delta_secs());
}

fn follow_camera(
    player_transform: Query<&Transform, (With<PlayerTransform>, Without<Camera3d>)>,
    mut camera_transform: Query<&mut Transform, (With<Camera3d>, Without<PlayerTransform>)>,
) {
    let Ok(player_transform) = player_transform.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_transform.single_mut() else {
        return;
    };

    let offset = Vec3::new(0.0, 5.0, 10.0);
    camera_transform.translation = player_transform.translation + offset;
    camera_transform.look_at(player_transform.translation, Vec3::Y);
}
