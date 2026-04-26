use bevy::prelude::*;

use crate::balance::Globals;
use crate::game_state::GameState;

use super::floor::spawn_floor;
use super::size::CurrentArenaSize;
use super::walls::spawn_walls;

pub fn register(app: &mut App) {
    app.init_resource::<CurrentArenaSize>()
        .add_systems(OnEnter(GameState::Playing), spawn_arena);
}

fn spawn_arena(
    mut commands: Commands,
    globals: Res<Globals>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    let radius = globals.arena_radius;
    arena_size.radius = radius;

    spawn_walls(&mut commands, radius);
    spawn_floor(&mut commands, &mut meshes, &mut materials, radius);
}
