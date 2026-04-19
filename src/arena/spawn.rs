use bevy::prelude::*;

use crate::balance::Globals;
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
    globals: Res<Globals>,
    camera_angle: Res<CameraAngle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    let width = globals.arena_width;
    let height = globals.arena_height;
    arena_size.width = width;
    arena_size.height = height;

    spawn_walls(&mut commands, width, height);
    spawn_floor(
        &mut commands,
        &mut meshes,
        &mut materials,
        width,
        height,
        camera_angle.degrees,
    );
}
