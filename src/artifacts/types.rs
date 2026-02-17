use bevy::prelude::*;

use rand::prelude::*;

use crate::stats::{ModifierDef, ModifierDefRaw, StatId, StatRange, StatRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ArtifactId(pub u32);

#[derive(Component)]
pub struct Artifact {
    pub artifact_id: ArtifactId,
    pub values: Vec<(StatId, f32)>,
}

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

    pub fn roll_values(&self, rng: &mut impl Rng) -> Vec<(StatId, f32)> {
        self.modifiers
            .iter()
            .flat_map(|m| m.stats.iter())
            .map(|sr| match sr {
                StatRange::Fixed { stat, value } => (*stat, *value),
                StatRange::Range { stat, min, max } => {
                    if (*max - *min).abs() < f32::EPSILON {
                        (*stat, *min)
                    } else {
                        (*stat, rng.random_range(*min..=*max))
                    }
                }
            })
            .collect()
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
