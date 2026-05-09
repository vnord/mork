use crate::components::enemy::Enemy;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
    }
}

fn spawn_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Enemy"),
        Enemy,
        Mesh3d(meshes.add(Capsule3d::new(0.4, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(5.0, 1.0, 5.0),
        RigidBody::Dynamic,
        Collider::capsule_y(0.6, 0.4),
    ));
}
