use bevy::prelude::*;

use super::types::ArtifactId;

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

    pub fn reset(&mut self, commands: &mut Commands) {
        for entity in self.slots.iter_mut().filter_map(|s| s.take()) {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Default)]
pub struct ShopOfferings(Vec<Entity>);

impl ShopOfferings {
    pub fn set(&mut self, entities: Vec<Entity>) {
        self.0 = entities;
    }

    pub fn get(&self, index: usize) -> Option<Entity> {
        self.0.get(index).copied()
    }

    pub fn position(&self, entity: Entity) -> Option<usize> {
        self.0.iter().position(|&e| e == entity)
    }

    pub fn remove(&mut self, index: usize) -> Entity {
        self.0.remove(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_slice(&self) -> &[Entity] {
        &self.0
    }

    pub fn clear(&mut self, commands: &mut Commands) {
        for entity in self.0.drain(..) {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Default)]
pub struct AvailableArtifacts(Vec<ArtifactId>);

impl AvailableArtifacts {
    pub fn new(ids: Vec<ArtifactId>) -> Self {
        Self(ids)
    }

    pub fn as_slice(&self) -> &[ArtifactId] {
        &self.0
    }
}

#[derive(Resource)]
pub struct RerollCost(u32);

impl RerollCost {
    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn reset_to(&mut self, base: u32) {
        self.0 = base;
    }
}

impl Default for RerollCost {
    fn default() -> Self {
        Self(1)
    }
}
