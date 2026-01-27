use bevy::prelude::*;
use rand::prelude::IndexedRandom;

use crate::abilities::ids::AbilityId;
use crate::abilities::AbilityRegistry;

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
            SpellSlot::Active => &["fireball", "flamethrower"],
            SpellSlot::Passive => &["meteor", "orbiting_orbs"],
            SpellSlot::Defensive => &["dash", "shield"],
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectedSpells {
    pub active: Option<AbilityId>,
    pub passive: Option<AbilityId>,
    pub defensive: Option<AbilityId>,
}

impl SelectedSpells {
    pub fn get(&self, slot: SpellSlot) -> Option<AbilityId> {
        match slot {
            SpellSlot::Active => self.active,
            SpellSlot::Passive => self.passive,
            SpellSlot::Defensive => self.defensive,
        }
    }

    pub fn set(&mut self, slot: SpellSlot, ability_id: AbilityId) {
        match slot {
            SpellSlot::Active => self.active = Some(ability_id),
            SpellSlot::Passive => self.passive = Some(ability_id),
            SpellSlot::Defensive => self.defensive = Some(ability_id),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.active.is_some() && self.passive.is_some() && self.defensive.is_some()
    }

    pub fn randomize(&mut self, ability_registry: &AbilityRegistry) {
        let mut rng = rand::rng();

        for slot in [SpellSlot::Active, SpellSlot::Passive, SpellSlot::Defensive] {
            let choices = slot.choices();
            if let Some(&choice) = choices.choose(&mut rng) {
                if let Some(id) = ability_registry.get_id(choice) {
                    self.set(slot, id);
                }
            }
        }
    }
}
