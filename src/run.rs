use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::blueprints::components::common::health::Health;
use crate::player::Player;
use crate::schedule::PostGameSet;
use crate::stats::{DeathEvent, death_system};
use crate::wave::WavePhase;

#[derive(Resource, Default)]
pub struct RunState {
    pub elapsed: f32,
    pub attempt: u32,
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .add_systems(OnEnter(WavePhase::Combat), init_run)
            .add_systems(
                Update,
                (tick_run, drain_stamina)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                PostUpdate,
                check_run_end
                    .after(death_system)
                    .in_set(PostGameSet),
            );
    }
}

fn init_run(mut run_state: ResMut<RunState>) {
    run_state.elapsed = 0.0;
    run_state.attempt += 1;
    info!("Starting run #{}", run_state.attempt);
}

fn tick_run(time: Res<Time>, mut run_state: ResMut<RunState>) {
    run_state.elapsed += time.delta_secs();
}

fn drain_stamina(
    time: Res<Time>,
    mut player_query: Query<&mut Health, With<Player>>,
) {
    for mut health in &mut player_query {
        health.current -= time.delta_secs();
    }
}

fn check_run_end(
    mut death_events: MessageReader<DeathEvent>,
    player_query: Query<(), With<Player>>,
    mut next_phase: ResMut<NextState<WavePhase>>,
    mut shop_timer: ResMut<crate::wave::ShopDelayTimer>,
    balance: Res<GameBalance>,
) {
    for event in death_events.read() {
        if player_query.contains(event.entity) {
            shop_timer.0 = Timer::from_seconds(balance.wave.shop_delay, TimerMode::Once);
            next_phase.set(WavePhase::ShopDelay);
        }
    }
}
