use serde::Deserialize;

use super::entity_def::{EntityDef, EntityDefRaw};
use crate::blueprints::expr::{ScalarExpr, ScalarExprRaw};
use crate::stats::StatRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintDefRaw {
    pub id: String,
    #[serde(default)]
    pub cooldown: Option<ScalarExprRaw>,
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct BlueprintDef {
    pub cooldown: ScalarExpr,
    pub entities: Vec<EntityDef>,
}

impl BlueprintDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> BlueprintDef {
        BlueprintDef {
            cooldown: self.cooldown.as_ref()
                .map(|c| c.resolve(stat_registry))
                .unwrap_or(ScalarExpr::Literal(f32::INFINITY)),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}
