use std::sync::Arc;
use bevy::prelude::*;

use super::{context::AbilityContext, trigger_def::ActionDef};
use crate::abilities::ids::AbilityId;

#[derive(Message, Clone)]
pub struct TriggerAbilityEvent {
    pub ability_id: AbilityId,
    pub context: AbilityContext,
}

#[derive(Message, Clone)]
pub struct ExecuteActionEvent {
    pub action: Arc<ActionDef>,
    pub context: AbilityContext,
}
