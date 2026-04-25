mod components;
mod plugins;
mod resources;
mod systems;

use bevy::prelude::*;
use bevy_rapier3d::prelude::NoUserData;

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
        .add_plugins(leafwing_input_manager::prelude::InputManagerPlugin::<
            input::Action,
        >::default())
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        })),
        bevy_rapier3d::prelude::RigidBody::Fixed,
        bevy_rapier3d::prelude::Collider::cuboid(10.0, 0.1, 10.0),
    ));
}

mod input {
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
    pub enum Action {
        Move,
        Dodge,
        LightAttack,
        HeavyAttack,
        Block,
        LockOn,
    }
}
