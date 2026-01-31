use bevy::prelude::*;

use super::{
    events::{ExecuteActionEvent, TriggerAbilityEvent, TriggerEvent},
    registry::AbilityRegistry,
};

pub fn ability_dispatcher(
    mut trigger_events: MessageReader<TriggerAbilityEvent>,
    mut action_events: MessageWriter<ExecuteActionEvent>,
    ability_registry: Res<AbilityRegistry>,
) {
    for event in trigger_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };

        let Some(root_trigger) = ability_def.get_trigger(ability_def.root_trigger) else {
            continue;
        };

        for &action_id in &root_trigger.actions {
            action_events.write(ExecuteActionEvent {
                ability_id: event.ability_id,
                action_id,
                context: event.context.clone(),
            });
        }
    }
}

pub fn trigger_dispatcher(
    mut trigger_events: MessageReader<TriggerEvent>,
    mut action_events: MessageWriter<ExecuteActionEvent>,
    ability_registry: Res<AbilityRegistry>,
) {
    for event in trigger_events.read() {
        let Some(ability) = ability_registry.get(event.ability_id) else {
            continue;
        };
        let Some(action) = ability.get_action(event.action_id) else {
            continue;
        };

        for &trigger_id in &action.triggers {
            let Some(trigger) = ability.get_trigger(trigger_id) else {
                continue;
            };

            if trigger.trigger_type == event.trigger_type {
                for &action_id in &trigger.actions {
                    action_events.write(ExecuteActionEvent {
                        ability_id: event.ability_id,
                        action_id,
                        context: event.context.clone(),
                    });
                }
            }
        }
    }
}
