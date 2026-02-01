use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::ParamValue;
use crate::abilities::ids::NodeTypeId;
use crate::abilities::events::ExecuteNodeEvent;
use crate::abilities::Target;
use crate::building_blocks::actions::ActionParams;
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
    mut action_events: MessageReader<ExecuteNodeEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
    mut cached_id: Local<Option<NodeTypeId>>,
) {
    let handler_id = *cached_id.get_or_insert_with(|| {
        node_registry.get_id("DamageParams")
            .expect("DamageParams not registered")
    });

    for event in action_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };
        let Some(node_def) = ability_def.get_node(event.node_id) else {
            continue;
        };

        if node_def.node_type != handler_id {
            continue;
        }

        let ActionParams::DamageParams(params) = node_def.params.unwrap_action() else {
            continue;
        };

        let Some(Target::Entity(target)) = event.context.target else {
            continue;
        };

        let caster_stats = stats_query
            .get(event.context.caster)
            .unwrap_or(&DEFAULT_STATS);
        let amount = params.amount.evaluate_f32(&caster_stats);

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
