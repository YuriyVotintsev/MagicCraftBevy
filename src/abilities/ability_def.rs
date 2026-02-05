use serde::Deserialize;

use crate::building_blocks::activators::ActivatorParamsRaw;
use super::entity_def::{EntityDef, EntityDefRaw};
use super::ActivatorParams;
use super::context::ProvidedFields;
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
        let activator_provided = self.activator.provided_fields();

        for entity_raw in &self.entities {
            validate_entity_fields(&self.id, activator_provided, entity_raw);
        }

        AbilityDef {
            activator_params: self.activator.resolve(stat_registry),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

fn validate_entity_fields(ability_id: &str, provided: ProvidedFields, entity_raw: &EntityDefRaw) {
    if let Some(count) = &entity_raw.count {
        let required = count.required_fields();
        if !provided.contains(required) {
            let missing = provided.missing(required);
            panic!(
                "Ability '{}': count expression requires fields [{}] not provided by activator/trigger",
                ability_id,
                missing.field_names().join(", ")
            );
        }
    }

    for component in &entity_raw.components {
        let (required, nested) = component.required_fields_and_nested();

        if !provided.contains(required) {
            let missing = provided.missing(required);
            panic!(
                "Ability '{}': component expression requires fields [{}] not provided",
                ability_id,
                missing.field_names().join(", ")
            );
        }

        if let Some((trigger_provided, nested_entities)) = nested {
            for nested_entity in nested_entities {
                validate_entity_fields(ability_id, trigger_provided, nested_entity);
            }
        }
    }
}
