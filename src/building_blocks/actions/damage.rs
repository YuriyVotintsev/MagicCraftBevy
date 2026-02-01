use std::collections::HashMap;
use bevy::prelude::*;
use crate::register_node;

use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::{ParamValue, ParamValueRaw, ParseNodeParams, resolve_param_value};
use crate::abilities::node::{NodeHandler, NodeKind};
use crate::abilities::ids::NodeTypeId;
use crate::abilities::events::ExecuteNodeEvent;
use crate::abilities::Target;
use crate::stats::{ComputedStats, PendingDamage, DEFAULT_STATS, StatRegistry};
use crate::schedule::GameSet;
use crate::GameState;

pub const DAMAGE: &str = "damage";

#[derive(Debug, Clone)]
pub struct DamageParams {
    pub amount: ParamValue,
}

impl ParseNodeParams for DamageParams {
    fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self {
        Self {
            amount: raw.get("amount")
                .map(|v| resolve_param_value(v, stat_registry))
                .expect("damage requires 'amount' parameter"),
        }
    }
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
        node_registry.get_id(DAMAGE)
            .expect("damage handler not registered")
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

        let params = node_def.params.unwrap_action().unwrap_damage();

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

#[derive(Default)]
pub struct DamageHandler;

impl NodeHandler for DamageHandler {
    fn name(&self) -> &'static str {
        DAMAGE
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Action
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

register_node!(DamageHandler, params: DamageParams, name: DAMAGE);
