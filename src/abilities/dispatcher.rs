use bevy::prelude::*;

use super::{
    events::{ExecuteActionEvent, TriggerAbilityEvent},
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

        for action_def in &ability_def.trigger.actions {
            action_events.write(ExecuteActionEvent {
                action: action_def.clone(),
                context: event.context.clone(),
            });
        }
    }
}
