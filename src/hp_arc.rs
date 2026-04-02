use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

use crate::arena::CameraAngle;
use crate::blueprints::components::common::health::Health;
use crate::blueprints::components::common::jump_walk_animation::animate as jump_animate;
use crate::blueprints::components::common::squish_walk_animation::animate as squish_animate;
use crate::blueprints::components::common::sprite::{CircleSprite, SquareSprite, TriangleSprite};
use crate::stats::{ComputedStats, Dead, StatRegistry};
use crate::Faction;

const INNER_RADIUS: f32 = 0.55;
const OUTER_RADIUS: f32 = 0.85;
const MAX_SWEEP: f32 = std::f32::consts::PI;
const CENTER_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
const ARC_Y_OFFSET: f32 = 0.0;
const ARC_SEGMENTS: u32 = 24;
const COLOR_FULL: (f32, f32, f32) = (0.4, 0.9, 0.3);
const COLOR_EMPTY: (f32, f32, f32) = (0.9, 0.2, 0.2);

#[derive(Component)]
struct HpArcSpawned {
    arc_entity: Entity,
}

#[derive(Component)]
struct HpArc {
    mesh_handle: Handle<Mesh>,
    material_handle: Handle<StandardMaterial>,
}

pub struct HpArcPlugin;

impl Plugin for HpArcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (spawn_hp_arc, update_hp_arc, orient_hp_arc)
                .chain()
                .after(jump_animate)
                .after(squish_animate),
        );
    }
}

fn build_arc_mesh(hp_fraction: f32) -> Mesh {
    let sweep = MAX_SWEEP * hp_fraction.clamp(0.01, 1.0);
    let start = CENTER_ANGLE - sweep / 2.0;

    let vert_count = (ARC_SEGMENTS as usize + 1) * 2;
    let mut positions = Vec::with_capacity(vert_count);
    let mut normals = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    for i in 0..=ARC_SEGMENTS {
        let t = i as f32 / ARC_SEGMENTS as f32;
        let angle = start + t * sweep;
        let (sin_a, cos_a) = angle.sin_cos();

        positions.push([cos_a * INNER_RADIUS, sin_a * INNER_RADIUS, 0.0]);
        positions.push([cos_a * OUTER_RADIUS, sin_a * OUTER_RADIUS, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([t, 0.0]);
        uvs.push([t, 1.0]);
    }

    let mut indices = Vec::with_capacity(ARC_SEGMENTS as usize * 6);
    for i in 0..ARC_SEGMENTS {
        let base = i * 2;
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

fn hp_to_color(hp_fraction: f32) -> Color {
    let t = hp_fraction.clamp(0.0, 1.0);
    Color::srgb(
        COLOR_EMPTY.0.lerp(COLOR_FULL.0, t),
        COLOR_EMPTY.1.lerp(COLOR_FULL.1, t),
        COLOR_EMPTY.2.lerp(COLOR_FULL.2, t),
    )
}

fn camera_rotation(camera_angle: &CameraAngle) -> Quat {
    Quat::from_rotation_x(camera_angle.degrees.to_radians() - std::f32::consts::FRAC_PI_2)
}

fn spawn_hp_arc(
    mut commands: Commands,
    camera_angle: Res<CameraAngle>,
    stat_registry: Res<StatRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (Entity, &Health, &ComputedStats, &Children, &Faction),
        (Changed<Health>, Without<HpArcSpawned>, Without<Dead>),
    >,
    sprite_query: Query<(), Or<(With<CircleSprite>, With<TriangleSprite>, With<SquareSprite>)>>,
) {
    let max_life_id = stat_registry.get("max_life");
    let max_stamina_id = stat_registry.get("max_stamina");

    for (entity, health, stats, children, faction) in &query {
        if *faction != Faction::Enemy {
            continue;
        }

        let max_life = max_life_id.map(|id| stats.get(id)).unwrap_or(0.0);
        let max_stamina = max_stamina_id.map(|id| stats.get(id)).unwrap_or(0.0);
        let max = max_life.max(max_stamina).max(1.0);

        if health.current >= max {
            continue;
        }

        let mut sprite_child = None;
        for child in children.iter() {
            if sprite_query.get(child).is_ok() {
                sprite_child = Some(child);
                break;
            }
        }
        let Some(sprite_child) = sprite_child else { continue };

        let hp_frac = health.current / max;
        let mesh_handle = meshes.add(build_arc_mesh(hp_frac));
        let material_handle = materials.add(StandardMaterial {
            base_color: hp_to_color(hp_frac),
            unlit: true,
            ..default()
        });

        let arc_entity = commands
            .spawn((
                HpArc {
                    mesh_handle: mesh_handle.clone(),
                    material_handle: material_handle.clone(),
                },
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Transform::from_translation(Vec3::new(0.0, ARC_Y_OFFSET, 0.0))
                    .with_rotation(camera_rotation(&camera_angle)),
            ))
            .id();

        commands.entity(sprite_child).add_child(arc_entity);
        commands.entity(entity).insert(HpArcSpawned { arc_entity });
    }
}

fn update_hp_arc(
    stat_registry: Res<StatRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (&Health, &ComputedStats, &HpArcSpawned),
        (Changed<Health>, Without<Dead>),
    >,
    arc_query: Query<&HpArc>,
) {
    let max_life_id = stat_registry.get("max_life");
    let max_stamina_id = stat_registry.get("max_stamina");

    for (health, stats, spawned) in &query {
        let Ok(hp_arc) = arc_query.get(spawned.arc_entity) else {
            continue;
        };

        let max_life = max_life_id.map(|id| stats.get(id)).unwrap_or(0.0);
        let max_stamina = max_stamina_id.map(|id| stats.get(id)).unwrap_or(0.0);
        let max = max_life.max(max_stamina).max(1.0);
        let hp_frac = (health.current / max).clamp(0.0, 1.0);

        if let Some(mesh) = meshes.get_mut(&hp_arc.mesh_handle) {
            *mesh = build_arc_mesh(hp_frac);
        }

        if let Some(material) = materials.get_mut(&hp_arc.material_handle) {
            material.base_color = hp_to_color(hp_frac);
        }
    }
}

fn orient_hp_arc(
    camera_angle: Res<CameraAngle>,
    parent_query: Query<&GlobalTransform>,
    mut arc_query: Query<(&ChildOf, &mut Transform), With<HpArc>>,
) {
    let desired = camera_rotation(&camera_angle);

    for (child_of, mut transform) in &mut arc_query {
        let Ok(parent_global) = parent_query.get(child_of.parent()) else {
            continue;
        };
        let parent_rot = parent_global.compute_transform().rotation;
        transform.rotation = parent_rot.inverse() * desired;
    }
}
