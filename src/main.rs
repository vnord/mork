#[allow(unused_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;

use bevy::prelude::*;
use bevy_rapier3d::parry::shape::SharedShape;
use bevy_rapier3d::prelude::{
    Collider, LockedAxes, NoUserData, PhysicsSet, QueryFilter, ReadRapierContext, RigidBody,
};
use bevy_third_person_camera::{
    CameraSyncSet, CustomGamepadSettings, ThirdPersonCamera, ThirdPersonCameraPlugin,
    ThirdPersonCameraTarget, Zoom,
};
use bevy_tnua::builtins::{
    TnuaBuiltinJump, TnuaBuiltinJumpConfig, TnuaBuiltinWalk, TnuaBuiltinWalkConfig,
};
use bevy_tnua::prelude::{
    TnuaConfig, TnuaController, TnuaControllerPlugin, TnuaScheme, TnuaUserControlsSystems,
};
use bevy_tnua_rapier3d::prelude::{TnuaRapier3dPlugin, TnuaRapier3dSensorShape};
use leafwing_input_manager::prelude::ActionState;
use mork::components::combat::PlayerMelee;
use mork::components::player::Player;
use mork::components::transform::PlayerTransform;
use mork::plugins::{combat::CombatPlugin, enemy::EnemyPlugin};
use mork::systems::input::{Action, default_input_map};
use mork::systems::movement::{
    calculate_camera_relative_movement_direction, movement_intent_from_axis,
};

const CAMERA_COLLISION_MARGIN: f32 = 0.2;

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
enum MovementScheme {
    Jump(TnuaBuiltinJump),
}

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
        .add_plugins(ThirdPersonCameraPlugin)
        .add_plugins(TnuaControllerPlugin::<MovementScheme>::new(Update))
        .add_plugins(TnuaRapier3dPlugin::new(Update))
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(EnemyPlugin)
        .configure_sets(PostUpdate, CameraSyncSet.after(PhysicsSet::StepSimulation))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player.in_set(TnuaUserControlsSystems))
        .add_systems(PostUpdate, prevent_camera_obstruction.after(CameraSyncSet))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut movement_configs: ResMut<Assets<MovementSchemeConfig>>,
) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        ThirdPersonCamera {
            cursor_lock_key: KeyCode::Escape,
            zoom: Zoom::new(6.0, 12.0),
            gamepad_settings: CustomGamepadSettings {
                sensitivity: Vec2::new(7.0, 4.0),
                ..default()
            },
            ..default()
        },
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

    commands.spawn((
        Name::new("Player"),
        Player,
        PlayerMelee::default(),
        PlayerTransform,
        ThirdPersonCameraTarget,
        default_input_map(),
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule_y(0.6, 0.4),
        TnuaController::<MovementScheme>::default(),
        TnuaConfig::<MovementScheme>(movement_configs.add(MovementSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                speed: 5.0,
                float_height: 1.0,
                max_slope: std::f32::consts::FRAC_PI_4,
                ..default()
            },
            jump: TnuaBuiltinJumpConfig {
                height: 2.0,
                ..default()
            },
        })),
        TnuaRapier3dSensorShape(SharedShape::cylinder(0.4, 0.35)),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
    ));
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn move_player(
    mut query: Query<(&ActionState<Action>, &mut TnuaController<MovementScheme>), With<Player>>,
    camera_transform: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    let Ok((action_state, mut controller)) = query.single_mut() else {
        return;
    };
    controller.initiate_action_feeding();

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

    controller.basis = TnuaBuiltinWalk {
        desired_motion: direction,
        desired_forward: Dir3::new(direction).ok(),
    };

    if action_state.pressed(&Action::Jump) {
        controller.action(MovementScheme::Jump(TnuaBuiltinJump::default()));
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn prevent_camera_obstruction(
    rapier_context: ReadRapierContext,
    player: Query<(Entity, &Transform), With<ThirdPersonCameraTarget>>,
    mut camera: Query<&mut Transform, (With<ThirdPersonCamera>, Without<ThirdPersonCameraTarget>)>,
) {
    let Ok(rapier_context) = rapier_context.single() else {
        return;
    };
    let Ok((player_entity, player_transform)) = player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let ray_origin = player_transform.translation;
    let camera_offset = camera_transform.translation - ray_origin;
    let camera_distance = camera_offset.length();
    let Some(ray_direction) = Dir3::new(camera_offset).ok() else {
        return;
    };

    let filter = QueryFilter::only_fixed()
        .exclude_sensors()
        .exclude_rigid_body(player_entity);

    let Some((_entity, time_of_impact)) =
        rapier_context.cast_ray(ray_origin, *ray_direction, camera_distance, true, filter)
    else {
        return;
    };

    let adjusted_distance = (time_of_impact - CAMERA_COLLISION_MARGIN).max(0.0);
    camera_transform.translation = ray_origin + *ray_direction * adjusted_distance;
}
