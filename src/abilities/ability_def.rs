use serde::Deserialize;

use super::components::{ComponentDef, ComponentDefRaw};
use super::entity_def::EntityDefRaw;
use super::context::ProvidedFields;
use crate::stats::StatRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct AbilityDefRaw {
    pub id: String,
    pub components: Vec<ComponentDefRaw>,
}

#[derive(Debug, Clone)]
pub struct AbilityDef {
    pub components: Vec<ComponentDef>,
}

impl AbilityDefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> AbilityDef {
        let has_activator = self.components.iter().any(|c| c.is_activator());
        if !has_activator {
            panic!("Ability '{}': must have at least one activator component", self.id);
        }

        for component in &self.components {
            if !component.is_activator() {
                panic!(
                    "Ability '{}': non-activator components must be nested inside activator's entities",
                    self.id
                );
            }
        }

        for component in &self.components {
            let activator_provided = component.provided_fields();
            let (_, nested) = component.required_fields_and_nested();

            if let Some((_, nested_entities)) = nested {
                for entity_raw in nested_entities {
                    validate_entity_fields(&self.id, activator_provided, entity_raw);
                }
            }
        }

        AbilityDef {
            components: self.components.iter().map(|c| c.resolve(stat_registry)).collect(),
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
