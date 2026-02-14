use bevy::prelude::*;

use super::types::{ArtifactDef, ArtifactId};

#[derive(Resource, Default)]
pub struct ArtifactRegistry {
    artifacts: Vec<ArtifactDef>,
}

impl ArtifactRegistry {
    pub fn register(&mut self, def: ArtifactDef) -> ArtifactId {
        let id = ArtifactId(self.artifacts.len() as u32);
        self.artifacts.push(def);
        id
    }

    pub fn get(&self, id: ArtifactId) -> Option<&ArtifactDef> {
        self.artifacts.get(id.0 as usize)
    }
}
