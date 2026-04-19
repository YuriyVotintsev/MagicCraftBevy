use bevy::prelude::*;

use crate::rune::{
    shop_grid_half_extent, BALL_RADIUS, SHOP_BALL_RING_RADIUS, SHOP_BALL_X,
};
use crate::wave::WavePhase;

use super::floor::disc_mesh;

const GRID_MARGIN: f32 = 60.0;
const BALLS_MARGIN: f32 = 50.0;

pub fn register(app: &mut App) {
    app.add_systems(OnEnter(WavePhase::Shop), spawn_shop_arena);
}

fn spawn_shop_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: crate::palette::color("background"),
        unlit: true,
        ..default()
    });

    let grid_extent = shop_grid_half_extent();
    let grid_radius = grid_extent.x.max(grid_extent.y) + GRID_MARGIN;

    commands.spawn((
        Name::new("ShopGridFloor"),
        Mesh3d(meshes.add(disc_mesh(grid_radius, 96))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(Vec3::new(0.0, -0.01, 0.0)),
        DespawnOnExit(WavePhase::Shop),
    ));

    let balls_radius = SHOP_BALL_RING_RADIUS + BALL_RADIUS + BALLS_MARGIN;

    commands.spawn((
        Name::new("ShopBallsFloor"),
        Mesh3d(meshes.add(disc_mesh(balls_radius, 64))),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(SHOP_BALL_X, -0.01, 0.0)),
        DespawnOnExit(WavePhase::Shop),
    ));
}
