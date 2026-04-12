use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::blueprints::components::mob::random_jump::RandomJump;
use crate::blueprints::components::mob::use_abilities::UseAbilities;

#[blueprint_component]
pub struct JumperAi {
    pub ability: String,
    #[raw(default = 3.0)]
    pub idle_duration: ScalarExpr,
    #[raw(default = 0.5)]
    pub jump_duration: ScalarExpr,
    #[raw(default = 0.5)]
    pub land_duration: ScalarExpr,
    #[raw(default = 350.0)]
    pub jump_distance: ScalarExpr,
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
        });
    }
}

fn jumper_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &JumperAi, &mut JumperAiState, &SpawnSource)>,
) {
    let dt = time.delta_secs();
    for (entity, ai, mut state, source) in &mut query {
        state.elapsed += dt;
        match state.phase {
            JumperPhase::Idle => {
                if state.elapsed >= ai.idle_duration && source.target.entity.is_some() {
                    state.phase = JumperPhase::Jump;
                    state.elapsed = 0.0;
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
                    commands.entity(entity).insert(UseAbilities {
                        abilities: vec![ai.ability.clone()],
                        cooldown: 999.0,
                        immediate: true,
                        max_range: None,
                    });
                }
            }
            JumperPhase::Land => {
                if state.elapsed >= ai.land_duration {
                    state.phase = JumperPhase::Idle;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<UseAbilities>();
                }
            }
        }
    }
}
