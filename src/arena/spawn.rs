use bevy::prelude::*;

use crate::balance::Globals;
use crate::game_state::GameState;
use crate::run::{RunState, StartWaveEvent};

use super::floor::spawn_floor;
use super::size::CurrentArenaSize;

pub const ARENA_GROWTH_PER_WAVE: f32 = 50.0;
pub const ARENA_MAX_RADIUS: f32 = 1000.0;

pub fn arena_radius_for_wave(base: f32, wave: u32) -> f32 {
    let bumped = wave.saturating_sub(1) as f32;
    (base + ARENA_GROWTH_PER_WAVE * bumped).min(ARENA_MAX_RADIUS)
}

pub fn register(app: &mut App) {
    app.init_resource::<CurrentArenaSize>()
        .add_systems(OnEnter(GameState::Playing), spawn_arena)
        .add_systems(
            Update,
            grow_arena_per_wave.run_if(in_state(GameState::Playing)),
        );
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

    spawn_floor(&mut commands, &mut meshes, &mut materials, radius);
}

fn grow_arena_per_wave(
    mut events: MessageReader<StartWaveEvent>,
    globals: Res<Globals>,
    run_state: Res<RunState>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    if events.read().last().is_none() {
        return;
    }
    let new_radius = arena_radius_for_wave(globals.arena_radius, run_state.wave);
    if (arena_size.radius - new_radius).abs() > 0.01 {
        arena_size.radius = new_radius;
    }
}
