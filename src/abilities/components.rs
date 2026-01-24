use bevy::prelude::*;

use super::ids::AbilityId;
use super::activator_def::ActivatorState;

#[derive(Debug, Clone)]
pub struct AbilityInstance {
    pub def_id: AbilityId,
    pub state: ActivatorState,
}

impl AbilityInstance {
    pub fn new(def_id: AbilityId) -> Self {
        Self {
            def_id,
            state: ActivatorState::new(),
        }
    }
}

#[derive(Component, Default)]
pub struct Abilities {
    pub list: Vec<AbilityInstance>,
}

impl Abilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, ability_id: AbilityId) {
        self.list.push(AbilityInstance::new(ability_id));
    }

    pub fn get(&self, ability_id: AbilityId) -> Option<&AbilityInstance> {
        self.list.iter().find(|a| a.def_id == ability_id)
    }

    pub fn get_mut(&mut self, ability_id: AbilityId) -> Option<&mut AbilityInstance> {
        self.list.iter_mut().find(|a| a.def_id == ability_id)
    }
}

use bevy::prelude::Vec3;

#[derive(Component, Default)]
pub struct AbilityInput {
    pub want_to_cast: Option<AbilityId>,
    pub target_direction: Option<Vec3>,
    pub target_point: Option<Vec3>,
}

impl AbilityInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_cast(&mut self, ability_id: AbilityId) {
        self.want_to_cast = Some(ability_id);
    }

    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.target_direction = Some(direction);
        self
    }

    pub fn with_point(mut self, point: Vec3) -> Self {
        self.target_point = Some(point);
        self
    }

    pub fn clear(&mut self) {
        self.want_to_cast = None;
        self.target_direction = None;
        self.target_point = None;
    }
}
