use bevy::prelude::*;

use crate::wave::WavePhase;

use super::camera::CameraAngle;
use super::floor::shop_floor_mesh;

pub const SHOP_ARENA_WIDTH: f32 = 2200.0;
pub const SHOP_ARENA_HEIGHT: f32 = 1600.0;

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

    commands.spawn((
        Name::new("ShopFloor"),
        Mesh3d(meshes.add(shop_floor_mesh(SHOP_ARENA_WIDTH, SHOP_ARENA_HEIGHT, 60.0, z_stretch))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: crate::palette::color("background"),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, -0.01, 0.0)),
        DespawnOnExit(WavePhase::Shop),
    ));
}
