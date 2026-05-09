use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, ExternalImpulse, QueryFilter, ReadRapierContext};
use leafwing_input_manager::prelude::ActionState;

use crate::components::combat::{HitFlash, PlayerMelee};
use crate::components::enemy::Enemy;
use crate::components::player::Player;
use crate::systems::input::Action;

pub const LIGHT_ATTACK_COOLDOWN_SECS: f32 = 0.42;
pub const LIGHT_ATTACK_HIT_RADIUS: f32 = 0.72;
pub const LIGHT_ATTACK_FORWARD_OFFSET: f32 = 1.0;
pub const LIGHT_ATTACK_UP_OFFSET: f32 = 0.35;
pub const LIGHT_ATTACK_IMPULSE: f32 = 5.5;
pub const LIGHT_HIT_FLASH_SECS: f32 = 0.14;
pub const LIGHT_HIT_FLASH_COLOR: Color = Color::srgb(1.0, 0.85, 0.4);

#[must_use]
pub fn xz_plane_push_dir(world_dir: Vec3) -> Vec3 {
    let mut v = world_dir;
    v.y = 0.0;
    let len_sq = v.length_squared();
    if len_sq > 1e-6 {
        v / len_sq.sqrt()
    } else {
        Vec3::NEG_Z
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn tick_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(Entity, &mut HitFlash, &MeshMaterial3d<StandardMaterial>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mat) in &mut q {
        flash.remaining -= dt;
        if flash.remaining > 0.0 {
            continue;
        }
        if let Some(m) = materials.get_mut(&mat.0) {
            m.base_color = flash.restore;
        }
        commands.entity(entity).remove::<HitFlash>();
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub fn player_light_attack(
    time: Res<Time>,
    rapier: ReadRapierContext,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut player: Query<
        (
            Entity,
            &ActionState<Action>,
            &mut PlayerMelee,
            &GlobalTransform,
        ),
        With<Player>,
    >,
    camera: Query<&GlobalTransform, (With<Camera3d>, Without<Player>)>,
    child_of: Query<&ChildOf>,
    children: Query<&Children>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    enemy_transforms: Query<&GlobalTransform, With<Enemy>>,
    enemy_marker: Query<(), With<Enemy>>,
) {
    let Ok(rapier) = rapier.single() else {
        return;
    };
    let Ok((player_entity, action, mut melee, player_tf)) = player.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    melee.light_cooldown = (melee.light_cooldown - dt).max(0.0);

    if !action.just_pressed(&Action::LightAttack) || melee.light_cooldown > 0.0 {
        return;
    }

    melee.light_cooldown = LIGHT_ATTACK_COOLDOWN_SECS;

    let Ok(cam_tf) = camera.single() else {
        return;
    };

    let swing_dir = xz_plane_push_dir(*cam_tf.forward());
    let origin = player_tf.translation()
        + Vec3::Y * LIGHT_ATTACK_UP_OFFSET
        + swing_dir * LIGHT_ATTACK_FORWARD_OFFSET;

    let hit_shape = Collider::ball(LIGHT_ATTACK_HIT_RADIUS);

    let mut hit_enemy_root: Option<Entity> = None;
    let mut best_dist_sq = f32::MAX;

    let filter = QueryFilter::default()
        .exclude_rigid_body(player_entity)
        .exclude_sensors();

    rapier.intersect_shape(
        origin,
        Quat::IDENTITY,
        (&hit_shape).into(),
        filter,
        |collider_entity| {
            if collider_entity == player_entity {
                return true;
            }
            let Some(enemy_root) =
                ancestor_with_enemy(collider_entity, &child_of, &enemy_marker)
            else {
                return true;
            };
            let Ok(enemy_tf) = enemy_transforms.get(enemy_root) else {
                return true;
            };
            let dist_sq = enemy_tf.translation().distance_squared(origin);
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                hit_enemy_root = Some(enemy_root);
            }
            true
        },
    );

    let Some(enemy_root) = hit_enemy_root else {
        return;
    };

    let Some(mesh_entity) =
        first_mesh_material_entity(enemy_root, &children, &mesh_materials)
    else {
        commands.entity(enemy_root).insert(ExternalImpulse {
            impulse: swing_dir * LIGHT_ATTACK_IMPULSE + Vec3::Y * 1.2,
            ..default()
        });
        return;
    };

    let Ok(mesh_mat) = mesh_materials.get(mesh_entity) else {
        return;
    };

    let restore = materials
        .get(&mesh_mat.0)
        .map_or(Color::WHITE, |m| m.base_color);

    if let Some(mat) = materials.get_mut(&mesh_mat.0) {
        mat.base_color = LIGHT_HIT_FLASH_COLOR;
    }

    commands.entity(mesh_entity).insert(HitFlash {
        remaining: LIGHT_HIT_FLASH_SECS,
        restore,
    });

    commands.entity(enemy_root).insert(ExternalImpulse {
        impulse: swing_dir * LIGHT_ATTACK_IMPULSE + Vec3::Y * 1.2,
        ..default()
    });
}

fn ancestor_with_enemy(
    mut entity: Entity,
    child_of: &Query<&ChildOf>,
    enemies: &Query<(), With<Enemy>>,
) -> Option<Entity> {
    loop {
        if enemies.contains(entity) {
            return Some(entity);
        }
        let Ok(link) = child_of.get(entity) else {
            return None;
        };
        entity = link.parent();
    }
}

fn first_mesh_material_entity(
    entity: Entity,
    children: &Query<&Children>,
    mesh_materials: &Query<&MeshMaterial3d<StandardMaterial>>,
) -> Option<Entity> {
    if mesh_materials.get(entity).is_ok() {
        return Some(entity);
    }
    let Ok(kids) = children.get(entity) else {
        return None;
    };
    for child in kids.iter() {
        if let Some(found) = first_mesh_material_entity(child, children, mesh_materials) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::xz_plane_push_dir;
    use bevy::math::Vec3;

    #[test]
    fn xz_plane_push_dir_flattens_and_normalizes() {
        let v = xz_plane_push_dir(Vec3::new(3.0, 4.0, 0.0));
        assert!((v.y).abs() < 1e-5);
        assert!((v.length() - 1.0).abs() < 1e-5);
        assert!((v.x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn xz_plane_push_dir_vertical_defaults_to_neg_z() {
        let v = xz_plane_push_dir(Vec3::Y);
        assert!((v - Vec3::NEG_Z).length() < 1e-5);
    }
}
