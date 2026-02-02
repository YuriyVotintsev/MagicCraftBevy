use bevy::prelude::*;

use super::{
    events::{ActivateAbilityEvent, ActionEventBase, NodeTriggerEvent},
    AbilityRegistry, NodeRegistry,
};
use super::node::NodeKind;
use crate::building_blocks::actions::ActionEventWriters;

fn collect_child_triggers(
    ability_def: &super::AbilityDef,
    node_def: &super::node::NodeDef,
    node_registry: &NodeRegistry,
) -> Vec<super::ids::NodeTypeId> {
    node_def
        .children
        .iter()
        .filter_map(|&child_id| {
            let child = ability_def.get_node(child_id)?;
            if node_registry.get_kind(child.node_type) == NodeKind::Trigger {
                Some(child.node_type)
            } else {
                None
            }
        })
        .collect()
}

pub fn node_ability_dispatcher(
    mut trigger_events: MessageReader<ActivateAbilityEvent>,
    mut action_writers: ActionEventWriters,
    ability_registry: Res<AbilityRegistry>,
    node_registry: Res<NodeRegistry>,
) {
    for event in trigger_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };

        for &node_id in &ability_def.root_action_nodes {
            let Some(node_def) = ability_def.get_node(node_id) else {
                continue;
            };

            let child_triggers = collect_child_triggers(ability_def, node_def, &node_registry);
            let base = ActionEventBase {
                ability_id: event.ability_id,
                node_id,
                context: event.context.clone(),
                child_triggers,
            };

            action_writers.dispatch(base, node_def.params.unwrap_action());
        }
    }
}

pub fn node_trigger_dispatcher(
    mut trigger_events: MessageReader<NodeTriggerEvent>,
    mut action_writers: ActionEventWriters,
    ability_registry: Res<AbilityRegistry>,
    node_registry: Res<NodeRegistry>,
) {
    for event in trigger_events.read() {
        let Some(ability) = ability_registry.get(event.ability_id) else {
            continue;
        };

        let Some(action_node) = ability.get_node(event.action_node_id) else {
            continue;
        };

        for &trigger_id in &action_node.children {
            let Some(trigger_node) = ability.get_node(trigger_id) else {
                continue;
            };

            if trigger_node.node_type == event.trigger_type {
                for &child_id in &trigger_node.children {
                    let Some(child_node) = ability.get_node(child_id) else {
                        continue;
                    };

                    let child_triggers = collect_child_triggers(ability, child_node, &node_registry);
                    let base = ActionEventBase {
                        ability_id: event.ability_id,
                        node_id: child_id,
                        context: event.context.clone(),
                        child_triggers,
                    };

                    action_writers.dispatch(base, child_node.params.unwrap_action());
                }
            }
        }
    }
}
