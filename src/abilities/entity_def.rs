use serde::Deserialize;

use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::stats::StatRegistry;
use super::components::{ComponentDef, ComponentDefRaw};

#[derive(Debug, Clone, Deserialize)]
pub struct EntityDefRaw {
    #[serde(default)]
    pub count: Option<ScalarExprRaw>,
    pub components: Vec<ComponentDefRaw>,
}

#[derive(Debug, Clone)]
pub struct EntityDef {
    pub count: Option<ScalarExpr>,
    pub components: Vec<ComponentDef>,
}

impl EntityDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> EntityDef {
        EntityDef {
            count: self.count.as_ref().map(|c| c.resolve(stat_registry)),
            components: self.components.iter().map(|c| c.resolve(stat_registry)).collect(),
        }
    }
}
