use bevy::prelude::*;

use crate::actors::{death_system, DeathEvent};
use crate::balance::GameBalance;
use crate::schedule::PostGameSet;
use crate::GameState;

use super::phase::WavePhase;

#[derive(Resource, Default)]
pub struct WaveState {
    pub spawned_count: u32,
    pub killed_count: u32,
    pub summoning_count: u32,
    pub max_concurrent: u32,
}

impl WaveState {
    pub fn new(balance: &crate::balance::WaveBalance) -> Self {
        Self {
            spawned_count: 0,
            killed_count: 0,
            summoning_count: 0,
            max_concurrent: balance.start_enemies,
        }
    }
}

#[derive(Component)]
pub struct WaveEnemy;

#[derive(Component)]
#[allow(dead_code)]
pub struct InvulnerableStack(pub u32);

pub fn register(app: &mut App) {
    app.init_resource::<WaveState>()
        .add_systems(OnEnter(WavePhase::Combat), reset_wave_state)
        .add_systems(
            PostUpdate,
            track_wave_kills
                .in_set(PostGameSet)
                .after(death_system)
                .run_if(in_state(GameState::Playing)),
        );
}

fn reset_wave_state(
    mut wave_state: ResMut<WaveState>,
    mut virtual_time: ResMut<Time<Virtual>>,
    balance: Res<GameBalance>,
) {
    *wave_state = WaveState::new(&balance.wave);
    virtual_time.unpause();
}

fn track_wave_kills(
    mut death_events: MessageReader<DeathEvent>,
    mut wave_state: ResMut<WaveState>,
    wave_enemy_query: Query<(), With<WaveEnemy>>,
) {
    for event in death_events.read() {
        if wave_enemy_query.contains(event.entity) {
            wave_state.killed_count += 1;
        }
    }
}

