use bevy::prelude::*;
use std::collections::HashMap;

use crate::abilities::registry::TriggerHandler;
use crate::abilities::ids::{AbilityId, ParamId, TriggerTypeId};
use crate::abilities::effect_def::ParamValue;

#[derive(Default)]
pub struct OnHitTrigger;

impl TriggerHandler for OnHitTrigger {
    fn name(&self) -> &'static str {
        "on_hit"
    }

    fn add_to_entity(
        &self,
        _commands: &mut Commands,
        _entity: Entity,
        _ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &crate::abilities::TriggerRegistry,
    ) {
    }

    fn register_systems(&self, _app: &mut App) {
    }
}

register_trigger!(OnHitTrigger);
