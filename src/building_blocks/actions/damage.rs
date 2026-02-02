use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::ParamValue;
use crate::abilities::Target;
use crate::building_blocks::actions::ExecuteDamageEvent;
use crate::stats::{ComputedStats, PendingDamage, DEFAULT_STATS};
use crate::schedule::GameSet;
use crate::GameState;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Action)]
pub struct DamageParams {
    pub amount: ParamValue,
}

fn execute_damage_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteDamageEvent>,
    stats_query: Query<&ComputedStats>,
) {
    for event in action_events.read() {
        let Some(Target::Entity(target)) = event.base.context.target else {
            continue;
        };

        let caster_stats = stats_query
            .get(event.base.context.caster)
            .unwrap_or(&DEFAULT_STATS);
        let amount = event.params.amount.evaluate_f32(&caster_stats);

        if let Ok(mut entity_commands) = commands.get_entity(target) {
            entity_commands.insert(PendingDamage(amount));
        }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        execute_damage_action
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

register_node!(DamageParams);
