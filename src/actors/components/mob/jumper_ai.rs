use bevy::prelude::*;

use crate::actors::abilities::{fire_ability, AbilitiesBalance, AbilityKind};
use crate::actors::SpawnSource;
use crate::actors::components::mob::random_jump::RandomJump;
use crate::stats::ComputedStats;

#[derive(Component)]
pub struct JumperAi {
    pub ability: String,
    pub idle_duration: f32,
    pub jump_duration: f32,
    pub land_duration: f32,
    pub jump_distance: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum JumperPhase {
    Idle,
    Jump,
    Land,
}

#[derive(Component)]
pub struct JumperAiState {
    phase: JumperPhase,
    elapsed: f32,
    ability_fired: bool,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_jumper_ai, jumper_ai_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
}

fn init_jumper_ai(
    mut commands: Commands,
    query: Query<Entity, Added<JumperAi>>,
) {
    for entity in &query {
        commands.entity(entity).insert(JumperAiState {
            phase: JumperPhase::Idle,
            elapsed: 0.0,
            ability_fired: false,
        });
    }
}

fn jumper_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    abilities_balance: Res<AbilitiesBalance>,
    stats_q: Query<&ComputedStats>,
    transforms: Query<&Transform>,
    mut query: Query<(Entity, &JumperAi, &mut JumperAiState, &SpawnSource), Without<crate::summoning::RiseFromGround>>,
) {
    let dt = time.delta_secs();
    for (entity, ai, mut state, source) in &mut query {
        state.elapsed += dt;
        match state.phase {
            JumperPhase::Idle => {
                if state.elapsed >= ai.idle_duration && source.target.entity.is_some() {
                    state.phase = JumperPhase::Jump;
                    state.elapsed = 0.0;
                    state.ability_fired = false;
                    commands.entity(entity).insert(RandomJump {
                        distance: ai.jump_distance,
                        duration: ai.jump_duration,
                    });
                }
            }
            JumperPhase::Jump => {
                if state.elapsed >= ai.jump_duration {
                    state.phase = JumperPhase::Land;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<RandomJump>();
                }
            }
            JumperPhase::Land => {
                if !state.ability_fired {
                    state.ability_fired = true;
                    if let Some(kind) = AbilityKind::from_id(&ai.ability) {
                        let caster_pos = transforms.get(entity).map(|t| crate::coord::to_2d(t.translation)).unwrap_or(Vec2::ZERO);
                        let caster_stats = stats_q.get(entity).ok();
                        fire_ability(
                            &mut commands,
                            kind,
                            entity,
                            caster_pos,
                            source.caster_faction,
                            source.target,
                            &abilities_balance,
                            caster_stats,
                        );
                    }
                }
                if state.elapsed >= ai.land_duration {
                    state.phase = JumperPhase::Idle;
                    state.elapsed = 0.0;
                }
            }
        }
    }
}
