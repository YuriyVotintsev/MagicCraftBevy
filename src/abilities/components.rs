use std::collections::HashMap;

use bevy::prelude::*;

use crate::faction::Faction;
pub use super::ids::{AbilityId, NodeDefId};

#[derive(Clone, Default)]
pub struct InputState {
    pub pressed: bool,
    pub just_pressed: bool,
    pub direction: Vec3,
}

#[derive(Component, Default)]
pub struct AbilityInputs {
    pub inputs: HashMap<AbilityId, InputState>,
}

impl AbilityInputs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, ability_id: AbilityId, state: InputState) {
        self.inputs.insert(ability_id, state);
    }

    pub fn get(&self, ability_id: AbilityId) -> Option<&InputState> {
        self.inputs.get(&ability_id)
    }

    pub fn clear(&mut self) {
        self.inputs.clear();
    }
}

#[derive(Component, Clone, Copy)]
pub struct AbilitySource {
    pub ability_id: AbilityId,
    pub node_id: NodeDefId,
    pub caster: Entity,
    pub caster_faction: Faction,
}

impl AbilitySource {
    pub fn new(ability_id: AbilityId, node_id: NodeDefId, caster: Entity, faction: Faction) -> Self {
        Self {
            ability_id,
            node_id,
            caster,
            caster_faction: faction,
        }
    }
}
