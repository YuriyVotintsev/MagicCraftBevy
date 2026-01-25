use bevy::prelude::*;
use std::collections::HashMap;

use super::types::MobDef;

#[derive(Resource, Default)]
pub struct MobRegistry {
    mobs: HashMap<String, MobDef>,
}

impl MobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, mob: MobDef) {
        self.mobs.insert(mob.name.clone(), mob);
    }

    pub fn get(&self, name: &str) -> Option<&MobDef> {
        self.mobs.get(name)
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MobDef)> {
        self.mobs.iter()
    }
}
