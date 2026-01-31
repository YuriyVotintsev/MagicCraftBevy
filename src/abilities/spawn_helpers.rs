use bevy::prelude::*;

use super::ids::AbilityId;
use super::node::{AbilityRegistry, NodeRegistry};

pub fn add_ability_trigger(
    commands: &mut Commands,
    entity: Entity,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
    node_registry: &NodeRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let Some(root_node) = ability_def.get_node(ability_def.root_node) else {
        return;
    };

    let Some(handler) = node_registry.get(root_node.node_type) else {
        warn!(
            "Unknown node type: {:?}",
            root_node.node_type
        );
        return;
    };

    handler.add_to_entity(
        commands,
        entity,
        ability_id,
        &root_node.params,
        node_registry,
    );
}

