use bevy::prelude::*;

use super::{
    events::{TriggerAbilityEvent, ExecuteNodeEvent, NodeTriggerEvent},
    AbilityRegistry,
};

pub fn node_ability_dispatcher(
    mut trigger_events: MessageReader<TriggerAbilityEvent>,
    mut execute_events: MessageWriter<ExecuteNodeEvent>,
    ability_registry: Res<AbilityRegistry>,
) {
    for event in trigger_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };

        for &node_id in &ability_def.root_action_nodes {
            execute_events.write(ExecuteNodeEvent {
                ability_id: event.ability_id,
                node_id,
                context: event.context.clone(),
            });
        }
    }
}

pub fn node_trigger_dispatcher(
    mut trigger_events: MessageReader<NodeTriggerEvent>,
    mut execute_events: MessageWriter<ExecuteNodeEvent>,
    ability_registry: Res<AbilityRegistry>,
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
                    execute_events.write(ExecuteNodeEvent {
                        ability_id: event.ability_id,
                        node_id: child_id,
                        context: event.context.clone(),
                    });
                }
            }
        }
    }
}
