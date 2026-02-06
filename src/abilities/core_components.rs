use std::collections::HashMap;

use bevy::prelude::*;

use crate::faction::Faction;
use super::context::TargetInfo;
pub use super::ids::AbilityId;

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
    pub caster: TargetInfo,
    pub caster_faction: Faction,
    pub source: TargetInfo,
    pub target: TargetInfo,
    pub index: usize,
    pub count: usize,
}
