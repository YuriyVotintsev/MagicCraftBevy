use std::fs;
use std::path::Path;

use crate::stats::StatRegistry;
use super::ids::TagId;
use super::registry::{AbilityRegistry, ActivatorRegistry, EffectRegistry};
use super::ability_def::{AbilityDef, AbilityDefRaw};
use super::activator_def::{ActivatorDef, ActivatorDefRaw};
use super::effect_def::{EffectDef, EffectDefRaw, ParamValue, ParamValueRaw};

fn resolve_param_value(
    raw: &ParamValueRaw,
    stat_registry: &StatRegistry,
    effect_registry: &mut EffectRegistry,
) -> ParamValue {
    match raw {
        ParamValueRaw::Float(v) => ParamValue::Float(*v),
        ParamValueRaw::Int(v) => ParamValue::Int(*v),
        ParamValueRaw::Bool(v) => ParamValue::Bool(*v),
        ParamValueRaw::String(v) => ParamValue::String(v.clone()),
        ParamValueRaw::Stat(name) => {
            let stat_id = stat_registry.get(name).unwrap_or_else(|| {
                panic!("Unknown stat '{}' in ability definition", name)
            });
            ParamValue::Stat(stat_id)
        }
        ParamValueRaw::Effect(raw_effect) => {
            let effect = resolve_effect_def(raw_effect, stat_registry, effect_registry);
            ParamValue::Effect(Box::new(effect))
        }
        ParamValueRaw::EffectList(raw_effects) => {
            let effects = raw_effects
                .iter()
                .map(|e| resolve_effect_def(e, stat_registry, effect_registry))
                .collect();
            ParamValue::EffectList(effects)
        }
    }
}

fn resolve_effect_def(
    raw: &EffectDefRaw,
    stat_registry: &StatRegistry,
    effect_registry: &mut EffectRegistry,
) -> EffectDef {
    let effect_type = effect_registry.get_id(&raw.effect_type).unwrap_or_else(|| {
        panic!("Unknown effect type '{}' in ability definition", raw.effect_type)
    });

    let params = raw
        .params
        .iter()
        .map(|(name, value)| {
            let param_id = effect_registry.get_or_insert_param_id(name);
            let resolved_value = resolve_param_value(value, stat_registry, effect_registry);
            (param_id, resolved_value)
        })
        .collect();

    EffectDef { effect_type, params }
}

fn resolve_activator_def(
    raw: &ActivatorDefRaw,
    stat_registry: &StatRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> ActivatorDef {
    let activator_type = activator_registry.get_id(&raw.activator_type).unwrap_or_else(|| {
        panic!("Unknown activator type '{}' in ability definition", raw.activator_type)
    });

    let params = raw
        .params
        .iter()
        .map(|(name, value)| {
            let param_id = effect_registry.get_or_insert_param_id(name);
            let resolved_value = resolve_param_value(value, stat_registry, effect_registry);
            (param_id, resolved_value)
        })
        .collect();

    ActivatorDef { activator_type, params }
}

fn resolve_ability_def(
    raw: &AbilityDefRaw,
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> AbilityDef {
    let id = ability_registry.allocate_id(&raw.id);

    let tags: Vec<TagId> = raw
        .tags
        .iter()
        .enumerate()
        .map(|(i, _)| TagId(i as u32))
        .collect();

    let activator = resolve_activator_def(&raw.activator, stat_registry, activator_registry, effect_registry);

    let effects = raw
        .effects
        .iter()
        .map(|e| resolve_effect_def(e, stat_registry, effect_registry))
        .collect();

    AbilityDef { id, tags, activator, effects }
}

pub fn load_abilities(
    abilities_dir: &str,
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) {
    let dir_path = Path::new(abilities_dir);

    if !dir_path.exists() {
        panic!("Abilities directory '{}' does not exist", abilities_dir);
    }

    let entries = fs::read_dir(dir_path).unwrap_or_else(|e| {
        panic!("Failed to read abilities directory '{}': {}", abilities_dir, e)
    });

    for entry in entries {
        let entry = entry.unwrap_or_else(|e| {
            panic!("Failed to read directory entry: {}", e)
        });

        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "ron") {
            let content = fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("Failed to read ability file '{}': {}", path.display(), e)
            });

            let raw: AbilityDefRaw = ron::from_str(&content).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse ability RON '{}': {}\nContent:\n{}",
                    path.display(), e, content
                )
            });

            let ability_def = resolve_ability_def(
                &raw,
                stat_registry,
                ability_registry,
                activator_registry,
                effect_registry,
            );

            ability_registry.register(ability_def);
        }
    }
}
