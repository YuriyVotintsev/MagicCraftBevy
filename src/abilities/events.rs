use bevy::prelude::*;

use super::context::AbilityContext;
use crate::abilities::ids::{AbilityId, NodeDefId, NodeTypeId};

#[derive(Message, Clone)]
pub struct TriggerAbilityEvent {
    pub ability_id: AbilityId,
    pub context: AbilityContext,
}

#[derive(Message, Clone)]
pub struct ExecuteNodeEvent {
    pub ability_id: AbilityId,
    pub node_id: NodeDefId,
    pub context: AbilityContext,
}

#[derive(Message, Clone)]
pub struct NodeTriggerEvent {
    pub ability_id: AbilityId,
    pub action_node_id: NodeDefId,
    pub trigger_type: NodeTypeId,
    pub context: AbilityContext,
}
