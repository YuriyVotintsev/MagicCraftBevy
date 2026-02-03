use serde::Deserialize;

use crate::building_blocks::activators::ActivatorParamsRaw;
use super::entity_def::{EntityDef, EntityDefRaw};
use super::ActivatorParams;
use crate::stats::StatRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct AbilityDefRaw {
    pub id: String,
    pub activator: ActivatorParamsRaw,
    #[serde(default)]
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub activator_params: ActivatorParams,
    pub entities: Vec<EntityDef>,
}

impl AbilityDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> AbilityDef {
        AbilityDef {
            activator_params: self.activator.resolve(stat_registry),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

impl AbilityDef {
    pub fn new(activator_params: ActivatorParams) -> Self {
        Self {
            activator_params,
            entities: vec![],
        }
    }
}
