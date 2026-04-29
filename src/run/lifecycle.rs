use bevy::prelude::*;

use crate::composite_scale::ScaleModifiers;
use crate::game_state::GameState;
use crate::wave::{CombatPhase, WaveEnemy};

use super::death::{PlayerDying, ShrinkToZero};

#[derive(Resource, Default)]
pub struct RunState {
    pub elapsed: f32,
    pub wave: u32,
}

#[derive(Resource, Debug)]
pub struct BreatherTimer(pub Timer);

#[derive(Message)]
pub struct StartWaveEvent;

pub fn wave_duration(_wave: u32) -> f32 {
    30.0
}

pub const BREATHER_DURATION: f32 = 7.0;

pub fn register(app: &mut App) {
    app.init_resource::<RunState>()
        .add_message::<StartWaveEvent>()
        .add_systems(OnEnter(GameState::Playing), enter_playing)
        .add_systems(
            Update,
            (
                tick_run.run_if(not(resource_exists::<BreatherTimer>)),
                check_combat_timeout
                    .run_if(not(resource_exists::<PlayerDying>))
                    .run_if(not(resource_exists::<BreatherTimer>)),
                tick_breather.run_if(resource_exists::<BreatherTimer>),
            )
                .run_if(in_state(CombatPhase::Running)),
        );
}

fn enter_playing(
    mut run_state: ResMut<RunState>,
    mut events: MessageWriter<StartWaveEvent>,
) {
    *run_state = RunState::default();
    run_state.wave = 1;
    events.write(StartWaveEvent);
}

fn tick_run(time: Res<Time>, mut run_state: ResMut<RunState>) {
    run_state.elapsed += time.delta_secs();
}

fn check_combat_timeout(
    mut commands: Commands,
    run_state: Res<RunState>,
    enemies: Query<(Entity, &Transform, Has<ScaleModifiers>), With<WaveEnemy>>,
) {
    if run_state.elapsed < wave_duration(run_state.wave) {
        return;
    }
    for (e, t, has) in &enemies {
        let mut ec = commands.entity(e);
        ec.insert(ShrinkToZero {
            elapsed: 0.0,
            duration: 0.5,
        });
        if !has {
            ec.insert(ScaleModifiers::default());
        }
        crate::particles::start_particles(
            &mut commands,
            "enemy_death",
            crate::coord::to_2d(t.translation),
        );
    }
    commands.insert_resource(BreatherTimer(Timer::from_seconds(
        BREATHER_DURATION,
        TimerMode::Once,
    )));
}

fn tick_breather(
    mut commands: Commands,
    time: Res<Time>,
    mut breather: ResMut<BreatherTimer>,
    mut run_state: ResMut<RunState>,
    mut events: MessageWriter<StartWaveEvent>,
) {
    if breather.0.tick(time.delta()).just_finished() {
        commands.remove_resource::<BreatherTimer>();
        run_state.wave += 1;
        run_state.elapsed = 0.0;
        events.write(StartWaveEvent);
    }
}
