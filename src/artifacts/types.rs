use crate::stats::{ModifierDef, ModifierDefRaw};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ArtifactId(pub u32);

pub struct ArtifactDef {
    pub name: String,
    pub price: u32,
    pub modifiers: Vec<ModifierDef>,
}

#[derive(serde::Deserialize)]
pub struct ArtifactDefRaw {
    pub id: String,
    pub name: String,
    pub price: u32,
    #[serde(default)]
    pub modifiers: Vec<ModifierDefRaw>,
}
