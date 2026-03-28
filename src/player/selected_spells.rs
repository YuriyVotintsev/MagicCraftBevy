use bevy::prelude::*;

use crate::blueprints::BlueprintId;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub enum SpellSlot {
    Active,
    Passive,
    Defensive,
}

#[derive(Resource, Default)]
pub struct SelectedSpells {
    pub active: Option<BlueprintId>,
    pub passive: Option<BlueprintId>,
    pub defensive: Option<BlueprintId>,
}

impl SelectedSpells {
    pub fn get(&self, slot: SpellSlot) -> Option<BlueprintId> {
        match slot {
            SpellSlot::Active => self.active,
            SpellSlot::Passive => self.passive,
            SpellSlot::Defensive => self.defensive,
        }
    }

    pub fn set(&mut self, slot: SpellSlot, blueprint_id: BlueprintId) {
        match slot {
            SpellSlot::Active => self.active = Some(blueprint_id),
            SpellSlot::Passive => self.passive = Some(blueprint_id),
            SpellSlot::Defensive => self.defensive = Some(blueprint_id),
        }
    }
}
