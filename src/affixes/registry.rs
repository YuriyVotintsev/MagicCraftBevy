use std::collections::HashMap;

use bevy::prelude::*;

use crate::player::SpellSlot;

use super::types::{AffixDef, AffixId};

#[derive(Resource, Default)]
pub struct AffixRegistry {
    affixes: Vec<AffixDef>,
    pools: HashMap<SpellSlot, Vec<AffixId>>,
}

impl AffixRegistry {
    pub fn register(&mut self, def: AffixDef, slot: SpellSlot) -> AffixId {
        let id = AffixId(self.affixes.len() as u32);
        self.affixes.push(def);
        self.pools.entry(slot).or_default().push(id);
        id
    }

    pub fn get(&self, id: AffixId) -> Option<&AffixDef> {
        self.affixes.get(id.0 as usize)
    }

    pub fn pool(&self, slot: SpellSlot) -> &[AffixId] {
        self.pools.get(&slot).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
