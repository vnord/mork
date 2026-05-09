#[cfg(debug_assertions)]
#[allow(clippy::single_component_path_imports, unused_imports)]
use bevy_dylib;

use bevy::gltf::GltfAssetLabel;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::{Affine2, Vec2};
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
use mork::components::character_visual::CharacterVisualSetup;
use mork::components::combat::PlayerMelee;
use mork::components::player::Player;
use mork::components::transform::PlayerTransform;
use mork::constants::{
    CAPSULE_HALF_HEIGHT, CAPSULE_RADIUS, PLAYER_VISUAL_OFFSET_Y, TNUA_FLOAT_HEIGHT,
};
use mork::plugins::{combat::CombatPlugin, enemy::EnemyPlugin};
use mork::systems::character_visual::{
    KAYKIT_IDLE_ANIMATION_INDEX, KAYKIT_LIGHT_ATTACK_ANIMATION_INDEX, KNIGHT_HIDDEN_NODES,
    character_visual_scene_ready,
};
use mork::systems::input::{Action, default_input_map};
use mork::systems::movement::{
    calculate_camera_relative_movement_direction, movement_intent_from_axis,
};

const CAMERA_COLLISION_MARGIN: f32 = 0.2;

const PLAYER_GLTF: &str = "models/third_party/Knight.glb";
const PLAYER_VISUAL_SCALE: f32 = 1.0;
const PLAYER_VISUAL_YAW: f32 = std::f32::consts::PI;

const FLOOR_ALBEDO: &str = "textures/floor/cobblestone_floor_08_diff_1k.png";
const FLOOR_UV_SCALE: f32 = 8.0;

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

#[allow(clippy::needless_pass_by_value)]
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    let floor_albedo = asset_server.load_with_settings(FLOOR_ALBEDO, |settings: &mut _| {
        *settings = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        };
    });

    commands.spawn((
        Name::new("Arena Floor"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(floor_albedo),
            perceptual_roughness: 0.95,
            metallic: 0.0,
            uv_transform: Affine2::from_scale(Vec2::splat(FLOOR_UV_SCALE)),
            ..default()
        })),
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(10.0, 0.1, 10.0),
    ));

    commands
        .spawn((
            Name::new("Player"),
            Player,
            PlayerMelee::default(),
            PlayerTransform,
            ThirdPersonCameraTarget,
            default_input_map(),
            Visibility::default(),
            Transform::from_xyz(0.0, 1.0, 0.0),
            RigidBody::Dynamic,
            Collider::capsule_y(CAPSULE_HALF_HEIGHT, CAPSULE_RADIUS),
            TnuaController::<MovementScheme>::default(),
            TnuaConfig::<MovementScheme>(movement_configs.add(MovementSchemeConfig {
                basis: TnuaBuiltinWalkConfig {
                    speed: 5.0,
                    float_height: TNUA_FLOAT_HEIGHT,
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
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Player visual"),
                    CharacterVisualSetup {
                        gltf_asset_path: PLAYER_GLTF,
                        idle_animation_index: KAYKIT_IDLE_ANIMATION_INDEX,
                        light_attack_animation_index: KAYKIT_LIGHT_ATTACK_ANIMATION_INDEX,
                        hidden_node_names: KNIGHT_HIDDEN_NODES,
                    },
                    SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(PLAYER_GLTF))),
                    Transform {
                        translation: Vec3::new(0.0, PLAYER_VISUAL_OFFSET_Y, 0.0),
                        rotation: Quat::from_rotation_y(PLAYER_VISUAL_YAW),
                        scale: Vec3::splat(PLAYER_VISUAL_SCALE),
                    },
                ))
                .observe(character_visual_scene_ready);
        });
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
