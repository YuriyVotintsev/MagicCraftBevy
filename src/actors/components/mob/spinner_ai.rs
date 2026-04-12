use bevy::prelude::*;

use crate::actors::SpawnSource;
use crate::actors::components::mob::spinner_charge::SpinnerCharge;
use crate::actors::components::mob::spinner_windup::SpinnerWindup;

#[derive(Component)]
pub struct SpinnerAi {
    pub idle_duration: f32,
    pub windup_duration: f32,
    pub charge_duration: f32,
    pub cooldown_duration: f32,
    pub charge_speed: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum SpinnerPhase {
    Idle,
    Windup,
    Charge,
    Cooldown,
}

#[derive(Component)]
pub struct SpinnerAiState {
    phase: SpinnerPhase,
    elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_spinner_ai, spinner_ai_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
}

fn init_spinner_ai(
    mut commands: Commands,
    query: Query<Entity, Added<SpinnerAi>>,
) {
    for entity in &query {
        commands.entity(entity).insert(SpinnerAiState {
            phase: SpinnerPhase::Idle,
            elapsed: 0.0,
        });
    }
}

fn spinner_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &SpinnerAi, &mut SpinnerAiState, &SpawnSource)>,
) {
    let dt = time.delta_secs();
    for (entity, ai, mut state, source) in &mut query {
        state.elapsed += dt;
        match state.phase {
            SpinnerPhase::Idle => {
                if state.elapsed >= ai.idle_duration && source.target.entity.is_some() {
                    state.phase = SpinnerPhase::Windup;
                    state.elapsed = 0.0;
                    commands.entity(entity).insert(SpinnerWindup {
                        duration: ai.windup_duration,
                        max_spin_speed: 10.0,
                        spike_growth_max: 3.0,
                        squish_min: 0.5,
                        contact_radius: 150.0,
                    });
                }
            }
            SpinnerPhase::Windup => {
                if state.elapsed >= ai.windup_duration {
                    state.phase = SpinnerPhase::Charge;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<SpinnerWindup>();
                    let target_entity = source.target.entity.unwrap_or(entity);
                    commands.entity(entity).insert(SpinnerCharge {
                        speed: ai.charge_speed,
                        max_duration: ai.charge_duration,
                        hit_radius: 150.0,
                        target: target_entity,
                    });
                }
            }
            SpinnerPhase::Charge => {
                if state.elapsed >= ai.charge_duration {
                    state.phase = SpinnerPhase::Cooldown;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<SpinnerCharge>();
                }
            }
            SpinnerPhase::Cooldown => {
                if state.elapsed >= ai.cooldown_duration {
                    state.phase = SpinnerPhase::Idle;
                    state.elapsed = 0.0;
                }
            }
        }
    }
}
