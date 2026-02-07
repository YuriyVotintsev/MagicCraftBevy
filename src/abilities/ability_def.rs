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

#[cfg(test)]
pub(crate) fn validate_entity_fields(
    ability_id: &str,
    provided: super::context::ProvidedFields,
    entity_raw: &EntityDefRaw,
    stat_names: &std::collections::HashSet<String>,
) -> Vec<String> {
    let mut errors = Vec::new();

    for name in entity_raw.base_stats.keys() {
        if !stat_names.contains(name) {
            errors.push(format!(
                "Ability '{}': unknown stat '{}' in base_stats",
                ability_id, name
            ));
        }
    }

    if let Some(count) = &entity_raw.count {
        let required = count.required_fields();
        if !provided.contains(required) {
            let missing = provided.missing(required);
            errors.push(format!(
                "Ability '{}': count expression requires fields [{}] not provided",
                ability_id,
                missing.field_names().join(", ")
            ));
        }
    }

    for component in &entity_raw.components {
        let (required, nested) = component.required_fields_and_nested();

        if !provided.contains(required) {
            let missing = provided.missing(required);
            errors.push(format!(
                "Ability '{}': component expression requires fields [{}] not provided",
                ability_id,
                missing.field_names().join(", ")
            ));
        }

        if let Some((trigger_provided, nested_entities)) = nested {
            for nested_entity in nested_entities {
                errors.extend(validate_entity_fields(ability_id, trigger_provided, nested_entity, stat_names));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::context::ProvidedFields;
    use super::super::components::ComponentDefRaw;
    use std::collections::{HashMap, HashSet};

    fn find_ron_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                find_ron_files(&path, out);
            } else {
                let name = path.file_name().unwrap().to_string_lossy();
                if name.ends_with(".ability.ron") || name.ends_with(".mob.ron") {
                    out.push(path);
                }
            }
        }
    }

    fn load_all_defs() -> (HashMap<String, AbilityDefRaw>, HashSet<String>) {
        let mut files = Vec::new();
        find_ron_files(std::path::Path::new("assets"), &mut files);

        let mut defs = HashMap::new();
        let mut mob_ids = HashSet::new();
        for path in &files {
            let content = std::fs::read_to_string(path).unwrap();
            let raw: AbilityDefRaw = ron::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));
            let name = path.file_name().unwrap().to_string_lossy();
            if name.ends_with(".mob.ron") {
                mob_ids.insert(raw.id.clone());
            }
            defs.insert(raw.id.clone(), raw);
        }
        (defs, mob_ids)
    }

    fn collect_use_abilities_refs(entity_raw: &super::super::entity_def::EntityDefRaw) -> Vec<String> {
        let mut refs = Vec::new();
        for component in &entity_raw.components {
            if let ComponentDefRaw::UseAbilities(raw) = component {
                refs.extend(raw.abilities.clone());
            }
        }
        if let Some(states) = &entity_raw.states {
            for state in states.states.values() {
                for component in &state.components {
                    if let ComponentDefRaw::UseAbilities(raw) = component {
                        refs.extend(raw.abilities.clone());
                    }
                }
            }
        }
        refs
    }

    fn collect_spawn_ability_refs(components: &[ComponentDefRaw]) -> Vec<String> {
        let mut refs = Vec::new();
        for component in components {
            match component {
                ComponentDefRaw::SpawnAbility(raw) => {
                    refs.push(raw.ability.clone());
                }
                ComponentDefRaw::OnDeath(raw) => {
                    for entity in &raw.entities {
                        refs.extend(collect_spawn_ability_refs(&entity.components));
                    }
                }
                _ => {}
            }
        }
        refs
    }

    fn load_stat_names() -> HashSet<String> {
        #[derive(serde::Deserialize)]
        struct StatsConfig {
            stat_ids: Vec<crate::stats::loader::StatDefRaw>,
            #[allow(dead_code)]
            calculators: Vec<crate::stats::loader::CalculatorDefRaw>,
        }
        let content = std::fs::read_to_string("assets/stats/config.stats.ron").unwrap();
        let config: StatsConfig = ron::from_str(&content).unwrap();
        config.stat_ids.iter().map(|s| s.name.clone()).collect()
    }

    #[test]
    fn validate_all_ability_fields() {
        let (defs, mob_ids) = load_all_defs();
        let stat_names = load_stat_names();
        let mut errors = Vec::new();
        let mut referenced = HashSet::new();

        let active_context = ProvidedFields::SOURCE_ENTITY
            .union(ProvidedFields::SOURCE_POSITION)
            .union(ProvidedFields::TARGET_DIRECTION);
        let passive_context = ProvidedFields::SOURCE_ENTITY
            .union(ProvidedFields::SOURCE_POSITION);
        let defensive_context = active_context;
        let mob_context = active_context;
        let spawn_context = ProvidedFields::SOURCE_ENTITY
            .union(ProvidedFields::SOURCE_POSITION);

        use crate::player::selected_spells::SpellSlot;
        for (slot, context) in [
            (SpellSlot::Active, active_context),
            (SpellSlot::Passive, passive_context),
            (SpellSlot::Defensive, defensive_context),
        ] {
            for &name in slot.choices() {
                referenced.insert(name.to_string());
                let Some(def) = defs.get(name) else {
                    errors.push(format!("Player {} ability '{}' not found in assets", slot.label(), name));
                    continue;
                };
                for entity_raw in &def.entities {
                    errors.extend(validate_entity_fields(name, context, entity_raw, &stat_names));
                }
            }
        }

        for def in defs.values() {
            for entity_raw in &def.entities {
                for name in entity_raw.base_stats.keys() {
                    if !stat_names.contains(name) {
                        errors.push(format!(
                            "Ability '{}': unknown stat '{}' in base_stats",
                            def.id, name
                        ));
                    }
                }

                let mob_abilities = collect_use_abilities_refs(entity_raw);
                for ability_name in &mob_abilities {
                    referenced.insert(ability_name.clone());
                    let Some(ability_def) = defs.get(ability_name) else {
                        errors.push(format!("Mob '{}': UseAbilities references '{}' not found", def.id, ability_name));
                        continue;
                    };
                    for entity in &ability_def.entities {
                        errors.extend(validate_entity_fields(ability_name, mob_context, entity, &stat_names));
                    }
                }

                let spawn_refs = collect_spawn_ability_refs(&entity_raw.components);
                for ability_name in &spawn_refs {
                    referenced.insert(ability_name.clone());
                    let Some(ability_def) = defs.get(ability_name) else {
                        errors.push(format!("Mob '{}': SpawnAbility references '{}' not found", def.id, ability_name));
                        continue;
                    };
                    for entity in &ability_def.entities {
                        errors.extend(validate_entity_fields(ability_name, spawn_context, entity, &stat_names));
                    }
                }
            }
        }

        for (id, _) in &defs {
            if !referenced.contains(id) && !mob_ids.contains(id) {
                errors.push(format!("Ability '{}' is defined but never referenced", id));
            }
        }

        if !errors.is_empty() {
            panic!("Validation errors:\n{}", errors.join("\n"));
        }
    }
}
