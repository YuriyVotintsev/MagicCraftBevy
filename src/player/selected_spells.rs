use bevy::prelude::*;
use rand::prelude::IndexedRandom;

use crate::blueprints::{BlueprintId, BlueprintRegistry};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SpellSlot {
    Active,
    Passive,
    Defensive,
}

impl SpellSlot {
    pub fn label(&self) -> &'static str {
        match self {
            SpellSlot::Active => "Active (LMB)",
            SpellSlot::Passive => "Passive (Auto)",
            SpellSlot::Defensive => "Defensive (Space)",
        }
    }

    pub fn choices(&self) -> &'static [&'static str] {
        match self {
            SpellSlot::Active => &["fireball", "flamethrower", "caustic_arrow", "galvanic_hammer"],
            SpellSlot::Passive => &["meteor", "orbiting_orbs"],
            SpellSlot::Defensive => &["dash", "shield"],
        }
    }
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

    pub fn is_complete(&self) -> bool {
        self.active.is_some() && self.passive.is_some() && self.defensive.is_some()
    }

    pub fn randomize(&mut self, blueprint_registry: &BlueprintRegistry) {
        let mut rng = rand::rng();

        for slot in [SpellSlot::Active, SpellSlot::Passive, SpellSlot::Defensive] {
            let choices = slot.choices();
            if let Some(&choice) = choices.choose(&mut rng) {
                if let Some(id) = blueprint_registry.get_id(choice) {
                    self.set(slot, id);
                }
            }
        }
    }
}
