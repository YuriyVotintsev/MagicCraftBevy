use bevy::prelude::*;

use crate::transition::{Transition, TransitionAction};
use crate::wave::WavePhase;

use super::death::PlayerDying;

pub const WAVE_COMBAT_DURATION: f32 = 30.0;

#[derive(Resource, Default)]
pub struct RunState {
    pub elapsed: f32,
    pub attempt: u32,
}

pub fn register(app: &mut App) {
    app.init_resource::<RunState>()
        .add_systems(OnEnter(WavePhase::Combat), init_run)
        .add_systems(
            Update,
            (
                tick_run,
                check_combat_timeout.run_if(not(resource_exists::<PlayerDying>)),
            )
                .run_if(in_state(WavePhase::Combat)),
        );
}

fn init_run(mut run_state: ResMut<RunState>) {
    run_state.elapsed = 0.0;
    run_state.attempt += 1;
    info!("Starting run #{}", run_state.attempt);
}

fn tick_run(time: Res<Time>, mut run_state: ResMut<RunState>) {
    run_state.elapsed += time.delta_secs();
}

fn check_combat_timeout(
    run_state: Res<RunState>,
    mut transition: ResMut<Transition>,
) {
    if run_state.elapsed >= WAVE_COMBAT_DURATION {
        transition.request(TransitionAction::Wave(WavePhase::Shop));
    }
}
