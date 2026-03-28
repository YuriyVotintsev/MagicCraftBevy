use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::money::PlayerMoney;
use crate::schedule::PostGameSet;
use crate::stats::{death_system, DeathEvent};
use crate::GameState;

#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(WavePhase = WavePhase::Combat)]
pub enum CombatPhase {
    #[default]
    Running,
    Paused,
}

#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
pub enum WavePhase {
    #[default]
    Combat,
    ShopDelay,
    Shop,
}

#[derive(Resource, Default)]
pub struct WaveState {
    pub spawned_count: u32,
    pub killed_count: u32,
    pub max_concurrent: u32,
}

impl WaveState {
    pub fn new(balance: &crate::balance::WaveBalance) -> Self {
        Self {
            spawned_count: 0,
            killed_count: 0,
            max_concurrent: balance.start_enemies,
        }
    }
}

#[derive(Component)]
pub struct WaveEnemy;

#[derive(Component)]
pub struct InvulnerableStack(pub u32);

impl InvulnerableStack {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn decrement(&mut self) -> bool {
        self.0 = self.0.saturating_sub(1);
        self.0 == 0
    }
}

#[derive(Resource)]
pub struct ShopDelayTimer(pub Timer);

impl Default for ShopDelayTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveState>()
            .init_resource::<PlayerMoney>()
            .init_resource::<ShopDelayTimer>()
            .add_systems(OnEnter(WavePhase::Combat), reset_wave_state)
            .add_systems(
                PostUpdate,
                track_wave_kills
                    .in_set(PostGameSet)
                    .after(death_system)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                shop_delay_tick.run_if(in_state(WavePhase::ShopDelay)),
            );
    }
}

fn reset_wave_state(
    mut wave_state: ResMut<WaveState>,
    mut shop_timer: ResMut<ShopDelayTimer>,
    mut virtual_time: ResMut<Time<Virtual>>,
    balance: Res<GameBalance>,
) {
    *wave_state = WaveState::new(&balance.wave);
    shop_timer.0 = Timer::from_seconds(balance.wave.shop_delay, TimerMode::Once);
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

fn shop_delay_tick(
    time: Res<Time>,
    mut timer: ResMut<ShopDelayTimer>,
    mut next_phase: ResMut<NextState<WavePhase>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next_phase.set(WavePhase::Shop);
    }
}
