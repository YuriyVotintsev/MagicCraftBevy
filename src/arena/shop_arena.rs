use bevy::prelude::*;

use crate::rune::{
    shop_grid_half_extent, BALL_RADIUS, SHOP_BALL_X, SHOP_BALL_Z_GAP, SHOP_SLOTS,
};
use crate::wave::WavePhase;

use super::camera::CameraAngle;
use super::floor::shop_floor_mesh;

const GRID_MARGIN: f32 = 40.0;
const BALLS_MARGIN: f32 = 40.0;
const FLOOR_CORNER_RADIUS: f32 = 60.0;
const BALLS_CORNER_RADIUS: f32 = 40.0;

pub fn register(app: &mut App) {
    app.add_systems(OnEnter(WavePhase::Shop), spawn_shop_arena);
}

fn spawn_shop_arena(
    mut commands: Commands,
    camera_angle: Res<CameraAngle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let elevation = (90.0 - camera_angle.degrees).to_radians();
    let z_stretch = 1.0 / elevation.sin();
    let material = materials.add(StandardMaterial {
        base_color: crate::palette::color("background"),
        unlit: true,
        ..default()
    });

    let grid_extent = shop_grid_half_extent();
    let grid_w = 2.0 * (grid_extent.x + GRID_MARGIN);
    let grid_h = 2.0 * (grid_extent.y + GRID_MARGIN);

    commands.spawn((
        Name::new("ShopGridFloor"),
        Mesh3d(meshes.add(shop_floor_mesh(grid_w, grid_h, FLOOR_CORNER_RADIUS, z_stretch))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(Vec3::new(0.0, -0.01, 0.0)),
        DespawnOnExit(WavePhase::Shop),
    ));

    let balls_z_span = (SHOP_SLOTS as f32 - 1.0) * SHOP_BALL_Z_GAP + BALL_RADIUS * 2.0;
    let balls_w = BALL_RADIUS * 2.0 + BALLS_MARGIN * 2.0;
    let balls_h = balls_z_span + BALLS_MARGIN * 2.0;

    commands.spawn((
        Name::new("ShopBallsFloor"),
        Mesh3d(meshes.add(shop_floor_mesh(balls_w, balls_h, BALLS_CORNER_RADIUS, z_stretch))),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(SHOP_BALL_X, -0.01, 0.0)),
        DespawnOnExit(WavePhase::Shop),
    ));
}
