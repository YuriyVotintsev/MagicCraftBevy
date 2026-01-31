use bevy::prelude::*;

use super::context::AbilityContext;
use crate::abilities::ids::{AbilityId, ActionDefId, TriggerTypeId};

#[derive(Message, Clone)]
pub struct TriggerAbilityEvent {
    pub ability_id: AbilityId,
    pub context: AbilityContext,
}

#[derive(Message, Clone)]
pub struct ExecuteActionEvent {
    pub ability_id: AbilityId,
    pub action_id: ActionDefId,
    pub context: AbilityContext,
}

#[derive(Message, Clone)]
pub struct TriggerEvent {
    pub ability_id: AbilityId,
    pub action_id: ActionDefId,
    pub trigger_type: TriggerTypeId,
    pub context: AbilityContext,
}
