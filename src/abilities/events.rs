use bevy::prelude::*;

use super::context::AbilityContext;
use crate::abilities::ids::AbilityId;

#[derive(Message, Clone)]
pub struct ActivateAbilityEvent {
    pub ability_id: AbilityId,
    pub context: AbilityContext,
}
