use bevy::prelude::*;

use crate::run::{CombatScoped, SkipDeathShrink};

use super::camera::CameraAngle;
use super::size::CurrentArenaSize;

#[derive(Component)]
pub(super) struct FloorParams {
    pub radius: f32,
}

pub fn register(app: &mut App) {
    app.add_systems(Update, update_floor_mesh);
}

pub(super) fn spawn_floor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    width: f32,
    height: f32,
    camera_angle_degrees: f32,
) {
    let floor_radius = 40.0;
    let elevation = (90.0 - camera_angle_degrees).to_radians();
    let z_stretch = 1.0 / elevation.sin();

    commands.spawn((
        Name::new("ArenaFloor"),
        FloorParams { radius: floor_radius },
        Mesh3d(meshes.add(rounded_floor_mesh(width, height, floor_radius, z_stretch))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: crate::palette::color("background"),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, -0.01, 0.0)),
        CombatScoped,
        SkipDeathShrink,
    ));
}

fn update_floor_mesh(
    camera_angle: Res<CameraAngle>,
    arena_size: Option<Res<CurrentArenaSize>>,
    query: Query<(&FloorParams, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(arena_size) = arena_size else { return };
    if !camera_angle.is_changed() && !arena_size.is_changed() {
        return;
    }
    let elevation = (90.0 - camera_angle.degrees).to_radians();
    let z_stretch = 1.0 / elevation.sin();
    for (params, mesh_handle) in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = rounded_floor_mesh(arena_size.width, arena_size.height, params.radius, z_stretch);
        }
    }
}

pub(super) fn shop_floor_mesh(width: f32, height: f32, radius: f32, z_stretch: f32) -> Mesh {
    rounded_floor_mesh(width, height, radius, z_stretch)
}

fn rounded_floor_mesh(width: f32, height: f32, radius: f32, z_stretch: f32) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};

    let hw = width / 2.0;
    let hh = height / 2.0;
    let r = radius.min(hw).min(hh);
    let rz = r * z_stretch;
    let segments = 8u32;

    let corner_centers = [
        [hw - r, -(hh - rz)],
        [hw - r, hh - rz],
        [-(hw - r), hh - rz],
        [-(hw - r), -(hh - rz)],
    ];

    let mut outline = Vec::new();
    for (i, center) in corner_centers.iter().enumerate() {
        let start = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::FRAC_PI_2;
        for j in 0..segments {
            let angle = start + std::f32::consts::FRAC_PI_2 * j as f32 / segments as f32;
            outline.push([center[0] + r * angle.cos(), center[1] + rz * angle.sin()]);
        }
    }

    let n = outline.len() as u32;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(1 + outline.len());
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(1 + outline.len());
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(1 + outline.len());
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for p in &outline {
        positions.push([p[0], 0.0, p[1]]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + p[0] / width, 0.5 + p[1] / height]);
    }

    for i in 0..n {
        indices.extend_from_slice(&[0, 1 + (i + 1) % n, 1 + i]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
