use bevy::prelude::*;

use crate::run::{CombatScoped, SkipDeathShrink};

use super::size::CurrentArenaSize;

#[derive(Component)]
pub(super) struct FloorParams;

pub fn register(app: &mut App) {
    app.add_systems(Update, update_floor_mesh);
}

pub(super) fn spawn_floor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    radius: f32,
) {
    commands.spawn((
        Name::new("ArenaFloor"),
        FloorParams,
        Mesh3d(meshes.add(disc_mesh(radius, 96))),
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
    arena_size: Option<Res<CurrentArenaSize>>,
    query: Query<&Mesh3d, With<FloorParams>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(arena_size) = arena_size else { return };
    if !arena_size.is_changed() {
        return;
    }
    for mesh_handle in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = disc_mesh(arena_size.radius, 96);
        }
    }
}

pub(super) fn disc_mesh(radius: f32, segments: u32) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};

    let n = segments.max(3);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(1 + n as usize);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(1 + n as usize);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(1 + n as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(n as usize * 3);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..n {
        let angle = std::f32::consts::TAU * i as f32 / n as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
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
