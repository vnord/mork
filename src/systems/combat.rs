use bevy::color::LinearRgba;
use bevy::math::primitives::Sphere;
use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalImpulse;
use leafwing_input_manager::prelude::ActionState;

use crate::components::combat::{HitBurst, HitFlash, PlayerMelee, PlayerWeaponBone};
use crate::components::enemy::Enemy;
use crate::components::player::Player;
use crate::constants::CAPSULE_RADIUS;
use crate::systems::input::Action;

#[cfg(debug_assertions)]
use bevy::gizmos::prelude::{GizmoPrimitive3d, Gizmos};
#[cfg(debug_assertions)]
use bevy::math::{Isometry3d, primitives::Capsule3d};

pub const LIGHT_ATTACK_COOLDOWN_SECS: f32 = 0.42;

pub const LIGHT_ATTACK_ACTIVE_START_SECS: f32 = 0.0;
pub const LIGHT_ATTACK_ACTIVE_END_SECS: f32 = 0.5;

pub const LIGHT_ATTACK_BLADE_HALF_LENGTH: f32 = 0.72;
pub const LIGHT_ATTACK_BLADE_RADIUS: f32 = 0.18;
pub const LIGHT_ATTACK_HAND_HEIGHT: f32 = 0.35;
pub const LIGHT_ATTACK_HAND_FORWARD: f32 = 0.55;

pub const LIGHT_ATTACK_LOCKON_RANGE: f32 = 6.0;
pub const LIGHT_ATTACK_PROXIMITY_MAX: f32 = 1.45;

pub const LIGHT_ATTACK_STRIKE_RELEASE_SECS: f32 = LIGHT_ATTACK_ACTIVE_END_SECS + 0.15;
pub const LIGHT_ATTACK_SWING_HALF_ARC_RAD: f32 = 0.62;

pub const LIGHT_ATTACK_IMPULSE: f32 = 5.5;
pub const LIGHT_HIT_FLASH_SECS: f32 = 0.55;
pub const LIGHT_HIT_FLASH_BASE_COLOR: Color = Color::srgb(1.0, 0.42, 0.52);
pub const LIGHT_HIT_FLASH_EMISSIVE: LinearRgba = LinearRgba::rgb(22.0, 9.0, 3.5);

pub const HIT_BURST_OUTER_RADIUS: f32 = 0.42;
pub const HIT_BURST_INNER_RADIUS: f32 = 0.14;
pub const HIT_BURST_OUTER_TTL_SECS: f32 = 0.28;
pub const HIT_BURST_INNER_TTL_SECS: f32 = 0.18;
pub const HIT_BURST_OUTER_EMISSIVE: LinearRgba = LinearRgba::rgb(55.0, 22.0, 6.0);
pub const HIT_BURST_INNER_EMISSIVE: LinearRgba = LinearRgba::rgb(90.0, 85.0, 75.0);

#[must_use]
fn distance_squared_point_to_segment(point: Vec3, segment_a: Vec3, segment_b: Vec3) -> f32 {
    let ab = segment_b - segment_a;
    let ap = point - segment_a;
    let ab_len_sq = ab.length_squared().max(1e-10);
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = segment_a + ab * t;
    point.distance_squared(closest)
}

#[must_use]
fn xz_distance_squared(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length_squared()
}

#[must_use]
fn nearest_enemy_basis_xz(
    player_entity: Entity,
    player_pos: Vec3,
    enemies: &Query<(Entity, &GlobalTransform), With<Enemy>>,
) -> Option<Vec3> {
    let r_sq = LIGHT_ATTACK_LOCKON_RANGE * LIGHT_ATTACK_LOCKON_RANGE;
    let mut best_d_sq = f32::MAX;
    let mut best_dir: Option<Vec3> = None;

    for (enemy_entity, enemy_tf) in enemies.iter() {
        if enemy_entity == player_entity {
            continue;
        }
        let epos = enemy_tf.translation();
        let d_sq = xz_distance_squared(player_pos, epos);
        if d_sq > r_sq || d_sq >= best_d_sq {
            continue;
        }
        best_d_sq = d_sq;
        best_dir = Some(xz_plane_push_dir(epos - player_pos));
    }

    best_dir.filter(|d| d.length_squared() > 1e-8)
}

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

pub struct LightAttackBladeFrame {
    pub blade_axis: Vec3,
    pub blade_mid: Vec3,
    pub blade_a: Vec3,
    pub blade_b: Vec3,
}

pub(crate) fn compute_light_attack_blade(
    player_entity: Entity,
    player_pos: Vec3,
    cam_tf: &GlobalTransform,
    elapsed_since_attack: f32,
    enemies: &Query<(Entity, &GlobalTransform), With<Enemy>>,
    weapon_tf: Option<&GlobalTransform>,
) -> Option<LightAttackBladeFrame> {
    if !(LIGHT_ATTACK_ACTIVE_START_SECS..=LIGHT_ATTACK_ACTIVE_END_SECS)
        .contains(&elapsed_since_attack)
    {
        return None;
    }

    let half = LIGHT_ATTACK_BLADE_HALF_LENGTH;

    if let Some(w) = weapon_tf {
        let blade_axis = Vec3::from(w.up());
        if blade_axis.length_squared() < 1e-10 {
            return None;
        }
        let blade_mid = w.translation() + blade_axis * half;
        return Some(LightAttackBladeFrame {
            blade_axis,
            blade_mid,
            blade_a: blade_mid - blade_axis * half,
            blade_b: blade_mid + blade_axis * half,
        });
    }

    let swing_basis = nearest_enemy_basis_xz(player_entity, player_pos, enemies)
        .unwrap_or_else(|| xz_plane_push_dir(*cam_tf.forward()));

    let denom = (LIGHT_ATTACK_ACTIVE_END_SECS - LIGHT_ATTACK_ACTIVE_START_SECS).max(1e-5);
    let strike_t =
        ((elapsed_since_attack - LIGHT_ATTACK_ACTIVE_START_SECS) / denom).clamp(0.0, 1.0);
    let swing_angle =
        -LIGHT_ATTACK_SWING_HALF_ARC_RAD + strike_t * (2.0 * LIGHT_ATTACK_SWING_HALF_ARC_RAD);
    let facing = Quat::from_axis_angle(Vec3::Y, swing_angle) * swing_basis;
    let blade_axis = xz_plane_push_dir(facing);

    let blade_mid =
        player_pos + Vec3::Y * LIGHT_ATTACK_HAND_HEIGHT + blade_axis * LIGHT_ATTACK_HAND_FORWARD;

    Some(LightAttackBladeFrame {
        blade_axis,
        blade_mid,
        blade_a: blade_mid - blade_axis * half,
        blade_b: blade_mid + blade_axis * half,
    })
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
            m.base_color = flash.restore_base_color;
            m.emissive = flash.restore_emissive;
        }
        commands.entity(entity).remove::<HitFlash>();
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn tick_hit_burst(
    time: Res<Time>,
    mut commands: Commands,
    mut bursts: Query<(Entity, &mut HitBurst)>,
) {
    let dt = time.delta_secs();
    for (entity, mut burst) in &mut bursts {
        burst.ttl -= dt;
        if burst.ttl <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub fn player_light_attack_input(
    time: Res<Time>,
    mut player: Query<(&ActionState<Action>, &mut PlayerMelee), With<Player>>,
) {
    let Ok((action, mut melee)) = player.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    melee.light_cooldown = (melee.light_cooldown - dt).max(0.0);

    if !action.just_pressed(&Action::LightAttack) || melee.light_cooldown > 0.0 {
        return;
    }

    melee.light_cooldown = LIGHT_ATTACK_COOLDOWN_SECS;
    melee.light_attack_anim_pending = true;
    melee.light_attack_started_secs = Some(time.elapsed_secs());
    melee.light_attack_hit_registered = false;
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
pub fn player_light_attack_hit_detection(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut player: Query<
        (
            Entity,
            &mut PlayerMelee,
            &GlobalTransform,
            Option<&PlayerWeaponBone>,
        ),
        With<Player>,
    >,
    transforms: Query<&GlobalTransform>,
    camera: Query<&GlobalTransform, (With<Camera3d>, Without<Player>)>,
    children: Query<&Children>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
) {
    let Ok((player_entity, mut melee, player_tf, weapon_bone)) = player.single_mut() else {
        return;
    };

    if melee.light_attack_hit_registered {
        return;
    }

    let Some(t0) = melee.light_attack_started_secs else {
        return;
    };

    let elapsed = time.elapsed_secs() - t0;
    if elapsed > LIGHT_ATTACK_STRIKE_RELEASE_SECS {
        melee.light_attack_started_secs = None;
        return;
    }

    let Ok(cam_tf) = camera.single() else {
        return;
    };

    let player_pos = player_tf.translation();
    let weapon_tf = weapon_bone.and_then(|bone| transforms.get(bone.0).ok());
    let Some(frame) = compute_light_attack_blade(
        player_entity,
        player_pos,
        cam_tf,
        elapsed,
        &enemies,
        weapon_tf,
    ) else {
        return;
    };

    let blade_a = frame.blade_a;
    let blade_b = frame.blade_b;
    let blade_mid = frame.blade_mid;
    let push_dir = xz_plane_push_dir(frame.blade_axis);

    let hit_tol = LIGHT_ATTACK_BLADE_RADIUS + CAPSULE_RADIUS + 0.28;
    let hit_tol_sq = hit_tol * hit_tol;
    let prox_sq = LIGHT_ATTACK_PROXIMITY_MAX * LIGHT_ATTACK_PROXIMITY_MAX;
    let melee_range_sq = 3.0_f32 * 3.0_f32;

    let mut hit_enemy_root: Option<Entity> = None;
    let mut best_score_sq = f32::MAX;

    for (enemy_entity, enemy_tf) in &enemies {
        if enemy_entity == player_entity {
            continue;
        }
        let p = enemy_tf.translation();
        let seg_d_sq = distance_squared_point_to_segment(p, blade_a, blade_b);
        let prox_d_sq = p.distance_squared(blade_mid);
        let in_melee = xz_distance_squared(player_pos, p) <= melee_range_sq;
        let segment_hit = seg_d_sq <= hit_tol_sq;
        let proximity_hit = in_melee && prox_d_sq <= prox_sq;

        if !(segment_hit || proximity_hit) {
            continue;
        }

        let score_sq = seg_d_sq.min(prox_d_sq);
        if score_sq < best_score_sq {
            best_score_sq = score_sq;
            hit_enemy_root = Some(enemy_entity);
        }
    }

    let Some(enemy_root) = hit_enemy_root else {
        return;
    };

    melee.light_attack_hit_registered = true;

    let fallback_pos = player_tf.translation();
    let enemy_center = enemies
        .get(enemy_root)
        .map_or(fallback_pos, |(_, tf)| tf.translation());

    apply_light_attack_hit_feedback(
        &mut commands,
        &mut meshes,
        &mut materials,
        &children,
        &mesh_materials,
        enemy_root,
        enemy_center,
        push_dir,
    );
}

#[allow(clippy::too_many_arguments)]
fn apply_light_attack_hit_feedback(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    children: &Query<&Children>,
    mesh_materials: &Query<&MeshMaterial3d<StandardMaterial>>,
    enemy_root: Entity,
    enemy_center: Vec3,
    push_dir: Vec3,
) {
    commands.entity(enemy_root).insert(ExternalImpulse {
        impulse: push_dir * LIGHT_ATTACK_IMPULSE + Vec3::Y * 1.2,
        ..default()
    });

    spawn_hit_bursts(commands, meshes, materials, enemy_center);

    let mut mesh_entities = Vec::new();
    collect_mesh_material_entities(enemy_root, children, mesh_materials, &mut mesh_entities);

    for mesh_entity in mesh_entities {
        let Ok(mesh_mat) = mesh_materials.get(mesh_entity) else {
            continue;
        };
        let Some(mat) = materials.get(&mesh_mat.0) else {
            continue;
        };
        let restore_base_color = mat.base_color;
        let restore_emissive = mat.emissive;

        if let Some(m) = materials.get_mut(&mesh_mat.0) {
            m.base_color = LIGHT_HIT_FLASH_BASE_COLOR;
            m.emissive = LIGHT_HIT_FLASH_EMISSIVE;
        }

        commands.entity(mesh_entity).insert(HitFlash {
            remaining: LIGHT_HIT_FLASH_SECS,
            restore_base_color,
            restore_emissive,
        });
    }
}

fn collect_mesh_material_entities(
    entity: Entity,
    children: &Query<&Children>,
    mesh_materials: &Query<&MeshMaterial3d<StandardMaterial>>,
    out: &mut Vec<Entity>,
) {
    if mesh_materials.get(entity).is_ok() {
        out.push(entity);
    }
    let Ok(kids) = children.get(entity) else {
        return;
    };
    for child in kids.iter() {
        collect_mesh_material_entities(child, children, mesh_materials, out);
    }
}

fn spawn_hit_bursts(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
) {
    let mut burst_pos = center;
    burst_pos.y += 0.25;

    let outer_mesh = meshes.add(Mesh::from(Sphere::new(HIT_BURST_OUTER_RADIUS)));
    let outer_mat = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: HIT_BURST_OUTER_EMISSIVE,
        emissive_exposure_weight: 0.0,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Name::new("Hit burst outer"),
        Mesh3d(outer_mesh),
        MeshMaterial3d(outer_mat),
        Transform::from_translation(burst_pos),
        HitBurst {
            ttl: HIT_BURST_OUTER_TTL_SECS,
        },
    ));

    let inner_mesh = meshes.add(Mesh::from(Sphere::new(HIT_BURST_INNER_RADIUS)));
    let inner_mat = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: HIT_BURST_INNER_EMISSIVE,
        emissive_exposure_weight: 0.0,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Name::new("Hit burst core"),
        Mesh3d(inner_mesh),
        MeshMaterial3d(inner_mat),
        Transform::from_translation(burst_pos + Vec3::Y * 0.06),
        HitBurst {
            ttl: HIT_BURST_INNER_TTL_SECS,
        },
    ));
}

#[cfg(debug_assertions)]
#[allow(clippy::needless_pass_by_value)]
pub fn debug_draw_light_attack_blade(
    time: Res<Time>,
    mut gizmos: Gizmos,
    player: Query<
        (
            Entity,
            &PlayerMelee,
            &GlobalTransform,
            Option<&PlayerWeaponBone>,
        ),
        With<Player>,
    >,
    transforms: Query<&GlobalTransform>,
    camera: Query<&GlobalTransform, (With<Camera3d>, Without<Player>)>,
    enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
) {
    let Ok((player_entity, melee, player_tf, weapon_bone)) = player.single() else {
        return;
    };

    let Some(t0) = melee.light_attack_started_secs else {
        return;
    };

    let elapsed = time.elapsed_secs() - t0;
    if elapsed > LIGHT_ATTACK_STRIKE_RELEASE_SECS {
        return;
    }

    let Ok(cam_tf) = camera.single() else {
        return;
    };

    let player_pos = player_tf.translation();
    let weapon_tf = weapon_bone.and_then(|bone| transforms.get(bone.0).ok());
    let Some(frame) = compute_light_attack_blade(
        player_entity,
        player_pos,
        cam_tf,
        elapsed,
        &enemies,
        weapon_tf,
    ) else {
        return;
    };

    let capsule = Capsule3d::new(
        LIGHT_ATTACK_BLADE_RADIUS,
        2.0 * LIGHT_ATTACK_BLADE_HALF_LENGTH,
    );
    let rotation = Quat::from_rotation_arc(Vec3::Y, frame.blade_axis);
    gizmos.primitive_3d(
        &capsule,
        Isometry3d::new(frame.blade_mid, rotation),
        Color::srgba(0.2, 1.0, 0.45, 0.9),
    );

    gizmos.line(
        frame.blade_a,
        frame.blade_b,
        Color::srgba(1.0, 1.0, 0.35, 0.95),
    );
}

#[cfg(test)]
mod tests {
    use super::{distance_squared_point_to_segment, xz_plane_push_dir};
    use bevy::math::Vec3;

    #[test]
    fn distance_squared_point_to_segment_midpoint() {
        let a = Vec3::ZERO;
        let b = Vec3::X * 4.0;
        let p = Vec3::X * 2.0;
        assert!(distance_squared_point_to_segment(p, a, b).abs() < 1e-5);
    }

    #[test]
    fn distance_squared_point_to_segment_perpendicular() {
        let a = Vec3::ZERO;
        let b = Vec3::X * 4.0;
        let p = Vec3::new(2.0, 3.0, 0.0);
        let d = distance_squared_point_to_segment(p, a, b).sqrt();
        assert!((d - 3.0).abs() < 1e-4);
    }

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
