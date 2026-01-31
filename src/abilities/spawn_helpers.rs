use bevy::prelude::*;

use super::ids::AbilityId;
use super::registry::{AbilityRegistry, TriggerRegistry};

pub fn add_ability_trigger(
    commands: &mut Commands,
    entity: Entity,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
    trigger_registry: &TriggerRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let Some(root_trigger) = ability_def.get_trigger(ability_def.root_trigger) else {
        return;
    };

    let Some(handler) = trigger_registry.get(root_trigger.trigger_type) else {
        warn!(
            "Unknown trigger type: {:?}",
            root_trigger.trigger_type
        );
        return;
    };

    handler.add_to_entity(
        commands,
        entity,
        ability_id,
        &root_trigger.params,
        trigger_registry,
    );
}
