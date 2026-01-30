use bevy::prelude::*;

use super::{
    events::{ExecuteEffectEvent, TriggerAbilityEvent},
    registry::AbilityRegistry,
};

pub fn ability_dispatcher(
    mut trigger_events: MessageReader<TriggerAbilityEvent>,
    mut effect_events: MessageWriter<ExecuteEffectEvent>,
    ability_registry: Res<AbilityRegistry>,
) {
    for event in trigger_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };

        for effect_def in &ability_def.effects {
            effect_events.write(ExecuteEffectEvent {
                effect: effect_def.clone(),
                context: event.context.clone(),
            });
        }
    }
}
