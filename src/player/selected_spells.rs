use bevy::prelude::*;

use crate::actors::abilities::AbilityKind;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Deserialize)]
pub enum SpellSlot {
    Active,
    Passive,
    Defensive,
}

#[derive(Resource, Default)]
pub struct SelectedSpells {
    pub active: Option<AbilityKind>,
    pub passive: Option<AbilityKind>,
    pub defensive: Option<AbilityKind>,
}

impl SelectedSpells {
    pub fn get(&self, slot: SpellSlot) -> Option<AbilityKind> {
        match slot {
            SpellSlot::Active => self.active,
            SpellSlot::Passive => self.passive,
            SpellSlot::Defensive => self.defensive,
        }
    }

    pub fn set(&mut self, slot: SpellSlot, kind: AbilityKind) {
        match slot {
            SpellSlot::Active => self.active = Some(kind),
            SpellSlot::Passive => self.passive = Some(kind),
            SpellSlot::Defensive => self.defensive = Some(kind),
        }
    }
}
