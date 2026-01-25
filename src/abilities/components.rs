use std::collections::HashMap;
use bevy::prelude::*;

use super::ids::AbilityId;
use super::activator_def::ActivatorState;

#[derive(Debug, Clone)]
pub struct AbilityInstance {
    pub state: ActivatorState,
}

impl AbilityInstance {
    pub fn new() -> Self {
        Self {
            state: ActivatorState::new(),
        }
    }
}

impl Default for AbilityInstance {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Default)]
pub struct Abilities {
    pub map: HashMap<AbilityId, AbilityInstance>,
}

impl Abilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, ability_id: AbilityId) {
        self.map.insert(ability_id, AbilityInstance::new());
    }

    pub fn get(&self, ability_id: AbilityId) -> Option<&AbilityInstance> {
        self.map.get(&ability_id)
    }

    pub fn get_mut(&mut self, ability_id: AbilityId) -> Option<&mut AbilityInstance> {
        self.map.get_mut(&ability_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (AbilityId, &AbilityInstance)> {
        self.map.iter().map(|(&id, inst)| (id, inst))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (AbilityId, &mut AbilityInstance)> {
        self.map.iter_mut().map(|(&id, inst)| (id, inst))
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
