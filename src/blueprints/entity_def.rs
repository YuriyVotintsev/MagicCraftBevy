use std::collections::HashMap;
use serde::Deserialize;

use crate::blueprints::expr::ScalarExpr;
use crate::expr::StatId;
use crate::expr::calc::CalcRegistry;

use super::components::{ComponentDef, ComponentDefRaw};

#[derive(Debug, Clone, Deserialize)]
pub struct EntityDefRaw {
    #[serde(default)]
    pub count: Option<String>,
    #[serde(default)]
    pub base_stats: HashMap<String, f32>,
    #[serde(default)]
    pub abilities: Vec<String>,
    pub components: Vec<ComponentDefRaw>,
}

#[derive(Debug, Clone)]
pub struct EntityDef {
    pub count: Option<ScalarExpr>,
    pub base_stats: HashMap<StatId, f32>,
    pub abilities: Vec<String>,
    pub components: Vec<ComponentDef>,
}

impl EntityDefRaw {
    pub fn resolve(&self, lookup: &dyn Fn(&str) -> Option<StatId>, calc_reg: &CalcRegistry) -> EntityDef {
        EntityDef {
            count: self.count.as_ref().map(|c| crate::expr::expand_parse_resolve_scalar(c, lookup, calc_reg)),
            base_stats: self.base_stats.iter()
                .filter_map(|(name, value)| {
                    lookup(name).map(|id| (id, *value))
                })
                .collect(),
            abilities: self.abilities.clone(),
            components: self.components.iter().map(|c| c.resolve(lookup, None, calc_reg)).collect(),
        }
    }
}
