use serde::Deserialize;

use super::entity_def::{EntityDef, EntityDefRaw};
use crate::blueprints::expr::ScalarExpr;
use crate::expr::StatId;
use crate::expr::calc::CalcRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintDefRaw {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cooldown: Option<String>,
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct BlueprintDef {
    pub name: Option<String>,
    pub cooldown: ScalarExpr,
    pub entities: Vec<EntityDef>,
}

impl BlueprintDefRaw {
    pub fn resolve(&self, lookup: &dyn Fn(&str) -> Option<StatId>, calc_reg: &CalcRegistry) -> BlueprintDef {
        BlueprintDef {
            name: self.name.clone(),
            cooldown: self.cooldown.as_ref()
                .map(|c| crate::expr::expand_parse_resolve_scalar(c, lookup, calc_reg))
                .unwrap_or(ScalarExpr::Literal(f32::INFINITY)),
            entities: self.entities.iter().map(|e| e.resolve(lookup, calc_reg)).collect(),
        }
    }
}
