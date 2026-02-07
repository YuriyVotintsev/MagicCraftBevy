use serde::Deserialize;

use super::entity_def::{EntityDef, EntityDefRaw};
use super::context::ProvidedFields;
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
        for entity_raw in &self.entities {
            validate_entity_fields(&self.id, ProvidedFields::ALL, entity_raw);
        }

        AbilityDef {
            cooldown: self.cooldown.as_ref()
                .map(|c| c.resolve(stat_registry))
                .unwrap_or(ScalarExpr::Literal(f32::INFINITY)),
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
                "Ability '{}': count expression requires fields [{}] not provided",
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
