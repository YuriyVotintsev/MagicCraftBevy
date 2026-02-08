use serde::Deserialize;

use super::entity_def::{EntityDef, EntityDefRaw};
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::stats::StatRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct AbilityDefRaw {
    pub id: String,
    #[serde(default)]
    pub cooldown: Option<ScalarExprRaw>,
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub cooldown: ScalarExpr,
    pub entities: Vec<EntityDef>,
}

impl AbilityDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> AbilityDef {
        AbilityDef {
            cooldown: self.cooldown.as_ref()
                .map(|c| c.resolve(stat_registry))
                .unwrap_or(ScalarExpr::Literal(f32::INFINITY)),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}
