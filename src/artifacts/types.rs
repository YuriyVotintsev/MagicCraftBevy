use bevy::prelude::*;

use crate::stats::{ModifierDef, ModifierDefRaw, StatRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ArtifactId(pub u32);

#[derive(Component)]
pub struct Artifact(pub ArtifactId);

#[derive(Debug, Clone, Copy)]
pub struct ArtifactEntity(pub Entity);

pub struct ArtifactDef {
    pub name: String,
    pub price: u32,
    pub modifiers: Vec<ModifierDef>,
}

impl ArtifactDef {
    pub fn sell_price(&self, percent: u32) -> u32 {
        self.price * percent / 100
    }
}

#[derive(serde::Deserialize)]
pub struct ArtifactDefRaw {
    pub id: String,
    pub name: String,
    pub price: u32,
    #[serde(default)]
    pub modifiers: Vec<ModifierDefRaw>,
}

impl ArtifactDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> ArtifactDef {
        ArtifactDef {
            name: self.name.clone(),
            price: self.price,
            modifiers: self.modifiers.iter().map(|m| m.resolve(stat_registry)).collect(),
        }
    }
}
