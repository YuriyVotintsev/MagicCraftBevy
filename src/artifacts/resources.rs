use bevy::prelude::*;

use super::types::ArtifactId;

#[derive(Component)]
pub struct Artifact(pub ArtifactId);

#[derive(Resource)]
pub struct PlayerArtifacts {
    pub slots: Vec<Option<Entity>>,
}

impl Default for PlayerArtifacts {
    fn default() -> Self {
        Self {
            slots: vec![None; 5],
        }
    }
}

impl PlayerArtifacts {
    pub fn buy(&mut self, entity: Entity) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
            *slot = Some(entity);
            true
        } else {
            false
        }
    }

    pub fn sell(&mut self, slot_index: usize) -> Option<Entity> {
        if slot_index < self.slots.len() {
            self.slots[slot_index].take()
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    pub fn equipped(&self) -> Vec<(usize, Entity)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.map(|e| (i, e)))
            .collect()
    }
}

#[derive(Resource, Default)]
pub struct ShopOfferings(pub Vec<Entity>);

#[derive(Resource, Default)]
pub struct AvailableArtifacts(pub Vec<ArtifactId>);

#[derive(Resource)]
pub struct RerollCost(pub u32);

impl Default for RerollCost {
    fn default() -> Self {
        Self(1)
    }
}
