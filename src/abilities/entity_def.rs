use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::stats::StatRegistry;
use super::components::{ComponentDef, ComponentDefRaw};

#[derive(Debug, Clone, Deserialize)]
pub struct EntityDefRaw {
    #[serde(default)]
    pub count: Option<ParamValueRaw>,
    pub components: Vec<ComponentDefRaw>,
}

#[derive(Debug, Clone)]
pub struct EntityDef {
    pub count: Option<ParamValue>,
    pub components: Vec<ComponentDef>,
}

impl EntityDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> EntityDef {
        EntityDef {
            count: self.count.as_ref().map(|c| resolve_param_value(c, stat_registry)),
            components: self.components.iter().map(|c| c.resolve(stat_registry)).collect(),
        }
    }
}
