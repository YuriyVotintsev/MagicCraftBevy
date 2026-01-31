use bevy::prelude::*;
use std::collections::HashMap;

use crate::abilities::node::{NodeHandler, NodeKind, NodeRegistry};
use crate::abilities::ids::{AbilityId, ParamId};
use crate::abilities::param::ParamValue;

#[derive(Default)]
pub struct OnHitTrigger;


impl NodeHandler for OnHitTrigger {
    fn name(&self) -> &'static str {
        "on_hit"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Trigger
    }

    fn add_to_entity(
        &self,
        _commands: &mut Commands,
        _entity: Entity,
        _ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &NodeRegistry,
    ) {
    }

    fn register_input_systems(&self, _app: &mut App) {
    }
}

register_node!(OnHitTrigger);
