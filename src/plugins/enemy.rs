use crate::components::character_visual::CharacterVisualSetup;
use crate::components::enemy::Enemy;
use crate::constants::{CAPSULE_HALF_HEIGHT, CAPSULE_RADIUS, ENEMY_VISUAL_OFFSET_Y};
use crate::systems::character_visual::{
    character_visual_scene_ready, KAYKIT_IDLE_ANIMATION_INDEX, ROGUE_HIDDEN_NODES,
};
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody};

const ENEMY_GLTF: &str = "models/third_party/Rogue.glb";
const ENEMY_VISUAL_YAW: f32 = std::f32::consts::PI;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_enemy(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Name::new("Enemy"),
            Enemy,
            Visibility::default(),
            Transform::from_xyz(5.0, 1.0, 5.0),
            RigidBody::Dynamic,
            Collider::capsule_y(CAPSULE_HALF_HEIGHT, CAPSULE_RADIUS),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Enemy visual"),
                    CharacterVisualSetup {
                        gltf_asset_path: ENEMY_GLTF,
                        idle_animation_index: KAYKIT_IDLE_ANIMATION_INDEX,
                        hidden_node_names: ROGUE_HIDDEN_NODES,
                    },
                    SceneRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset(ENEMY_GLTF)),
                    ),
                    Transform {
                        translation: Vec3::new(0.0, ENEMY_VISUAL_OFFSET_Y, 0.0),
                        rotation: Quat::from_rotation_y(ENEMY_VISUAL_YAW),
                        scale: Vec3::splat(1.0),
                    },
                ))
                .observe(character_visual_scene_ready);
        });
}
