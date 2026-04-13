use bevy::prelude::*;

pub mod coin;
pub mod death;
pub mod money;

pub use death::PlayerDying;

use crate::actors::combat::Health;
use crate::actors::player::Player;
use crate::wave::WavePhase;

#[derive(Resource, Default)]
pub struct RunState {
    pub elapsed: f32,
    pub attempt: u32,
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        death::register(app);
        coin::register(app);
        money::register(app);
        app.init_resource::<RunState>()
            .add_systems(OnEnter(WavePhase::Combat), init_run)
            .add_systems(
                Update,
                (
                    tick_run,
                    drain_life.run_if(not(resource_exists::<PlayerDying>)),
                )
                    .run_if(in_state(WavePhase::Combat)),
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

fn drain_life(
    time: Res<Time>,
    mut player_query: Query<&mut Health, With<Player>>,
) {
    for mut health in &mut player_query {
        health.current -= time.delta_secs();
    }
}
