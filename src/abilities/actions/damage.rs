use bevy::prelude::*;

use crate::abilities::registry::{ActionHandler, ActionRegistry};
use crate::abilities::events::ExecuteActionEvent;
use crate::stats::{ComputedStats, PendingDamage};
use crate::schedule::GameSet;
use crate::GameState;

fn execute_damage_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteActionEvent>,
    action_registry: Res<ActionRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in action_events.read() {
        let Some(handler_id) = action_registry.get_id("damage") else {
            continue;
        };
        if event.action.action_type != handler_id {
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
        let Some(amount) = event.action.get_f32("amount", &caster_stats, &action_registry) else {
            continue;
        };

        if let Ok(mut entity_commands) = commands.get_entity(target) {
            entity_commands.insert(PendingDamage(amount));
        }
    }
}

#[derive(Default)]
pub struct DamageHandler;

impl ActionHandler for DamageHandler {
    fn name(&self) -> &'static str {
        "damage"
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_damage_action
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_action!(DamageHandler);
