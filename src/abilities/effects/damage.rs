use bevy::prelude::*;

use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::abilities::events::ExecuteEffectEvent;
use crate::stats::{ComputedStats, PendingDamage};
use crate::schedule::GameSet;
use crate::GameState;

fn execute_damage_effect(
    mut commands: Commands,
    mut effect_events: MessageReader<ExecuteEffectEvent>,
    effect_registry: Res<EffectRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in effect_events.read() {
        let Some(handler_id) = effect_registry.get_id("damage") else {
            continue;
        };
        if event.effect.effect_type != handler_id {
            continue;
        }

        let Some(target) = event.context.get_param_entity("target") else {
            continue;
        };

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();
        let Some(amount) = event.effect.get_f32("amount", &caster_stats, &effect_registry) else {
            continue;
        };

        if let Ok(mut entity_commands) = commands.get_entity(target) {
            entity_commands.insert(PendingDamage(amount));
        }
    }
}

#[derive(Default)]
pub struct DamageHandler;

impl EffectHandler for DamageHandler {
    fn name(&self) -> &'static str {
        "damage"
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_damage_effect
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_effect!(DamageHandler);
