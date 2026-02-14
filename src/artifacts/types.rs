use std::collections::HashMap;

use crate::stats::StatId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ArtifactId(pub u32);

impl From<u32> for ArtifactId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<ArtifactId> for u32 {
    fn from(id: ArtifactId) -> Self {
        id.0
    }
}

pub struct ArtifactModifier {
    pub stat: StatId,
    pub value: f32,
    pub name: String,
}

pub struct ArtifactDef {
    pub name: String,
    pub price: u32,
    pub modifiers: Vec<ArtifactModifier>,
}

#[derive(serde::Deserialize)]
pub struct ArtifactDefRaw {
    pub id: String,
    pub name: String,
    pub price: u32,
    #[serde(default)]
    pub modifiers: HashMap<String, f32>,
}
