use std::collections::HashMap;

use bevy::prelude::*;

use crate::player::SpellSlot;

use super::components::Affixes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrbId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum OrbBehavior {
    Alteration,
    Chaos,
    Augmentation,
}

pub struct OrbDef {
    pub name: String,
    pub description: String,
    pub price: u32,
    pub behavior: OrbBehavior,
}

#[derive(serde::Deserialize)]
pub struct OrbDefRaw {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: u32,
    pub orb_type: OrbBehavior,
}

#[derive(Resource, Default)]
pub struct OrbRegistry {
    orbs: Vec<OrbDef>,
    name_to_id: HashMap<String, OrbId>,
}

impl OrbRegistry {
    pub fn register(&mut self, id_str: &str, def: OrbDef) -> OrbId {
        let id = OrbId(self.orbs.len() as u32);
        self.name_to_id.insert(id_str.to_string(), id);
        self.orbs.push(def);
        id
    }

    pub fn get(&self, id: OrbId) -> Option<&OrbDef> {
        self.orbs.get(id.0 as usize)
    }

    pub fn all_ids(&self) -> Vec<OrbId> {
        (0..self.orbs.len() as u32).map(OrbId).collect()
    }
}

#[derive(Resource, Default)]
pub enum OrbFlowState {
    #[default]
    None,
    SelectSlot {
        orb_id: OrbId,
    },
    Preview {
        slot_entity: Entity,
        slot_type: SpellSlot,
        original: Affixes,
        preview: Affixes,
        rerolled: [bool; 6],
    },
}
