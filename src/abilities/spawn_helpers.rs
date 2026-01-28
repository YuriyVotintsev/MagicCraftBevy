use bevy::prelude::*;

use super::ids::AbilityId;
use super::registry::{AbilityRegistry, ActivatorRegistry};

pub fn add_ability_activator(
    commands: &mut Commands,
    entity: Entity,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
    activator_registry: &ActivatorRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let Some(handler) = activator_registry.get(ability_def.activator.activator_type) else {
        warn!(
            "Unknown activator type: {:?}",
            ability_def.activator.activator_type
        );
        return;
    };

    handler.add_to_entity(
        commands,
        entity,
        ability_id,
        &ability_def.activator.params,
        activator_registry,
    );
}
