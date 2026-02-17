use std::collections::HashMap;

use bevy::prelude::*;

use super::types::{ArtifactDef, ArtifactId};

#[derive(Resource, Default)]
pub struct ArtifactRegistry {
    artifacts: Vec<ArtifactDef>,
    name_to_id: HashMap<String, ArtifactId>,
}

impl ArtifactRegistry {
    pub fn register(&mut self, id_str: &str, def: ArtifactDef) -> ArtifactId {
        let id = ArtifactId(self.artifacts.len() as u32);
        self.name_to_id.insert(id_str.to_string(), id);
        self.artifacts.push(def);
        id
    }

    pub fn get(&self, id: ArtifactId) -> Option<&ArtifactDef> {
        self.artifacts.get(id.0 as usize)
    }

    pub fn get_id(&self, name: &str) -> Option<ArtifactId> {
        self.name_to_id.get(name).copied()
    }
}
