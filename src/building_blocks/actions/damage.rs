use bevy::prelude::*;
use crate::register_node;

use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::node::{NodeHandler, NodeKind};
use crate::abilities::events::ExecuteNodeEvent;
use crate::abilities::Target;
use crate::stats::{ComputedStats, PendingDamage};
use crate::schedule::GameSet;
use crate::GameState;

fn execute_damage_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteNodeEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    let Some(handler_id) = node_registry.get_id("damage") else {
        return;
    };

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

        let Some(Target::Entity(target)) = event.context.target else {
            continue;
        };

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();
        let Some(amount) = node_def.get_f32("amount", &caster_stats, &node_registry) else {
            continue;
        };

        if let Ok(mut entity_commands) = commands.get_entity(target) {
            entity_commands.insert(PendingDamage(amount));
        }
    }
}

#[derive(Default)]
pub struct DamageHandler;

impl NodeHandler for DamageHandler {
    fn name(&self) -> &'static str {
        "damage"
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

register_node!(DamageHandler);
