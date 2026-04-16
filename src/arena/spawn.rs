use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::wave::WavePhase;

use super::camera::CameraAngle;
use super::floor::spawn_floor;
use super::size::CurrentArenaSize;
use super::walls::spawn_walls;

pub fn register(app: &mut App) {
    app.init_resource::<CurrentArenaSize>()
        .add_systems(OnEnter(WavePhase::Combat), spawn_arena);
}

fn spawn_arena(
    mut commands: Commands,
    balance: Res<GameBalance>,
    camera_angle: Res<CameraAngle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    let arena = &balance.arena;
    arena_size.width = arena.width;
    arena_size.height = arena.height;

    spawn_walls(&mut commands, arena.width, arena.height);
    spawn_floor(
        &mut commands,
        &mut meshes,
        &mut materials,
        arena.width,
        arena.height,
        camera_angle.degrees,
    );
}
