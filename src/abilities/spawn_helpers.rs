use bevy::prelude::*;

use super::ids::AbilityId;
use super::registry::{AbilityRegistry, ActivatorRegistry};
use super::effect_def::ParamValue;
use super::activators::{
    OnInputActivations, PassiveActivations, WhileHeldActivations, IntervalActivations,
};

pub fn add_ability_activator(
    commands: &mut Commands,
    entity: Entity,
    ability_id: AbilityId,
    ability_registry: &AbilityRegistry,
    activator_registry: &ActivatorRegistry,
) {
    let Some(ability_def) = ability_registry.get(ability_id) else {
        return;
    };

    let Some(activator_name) = activator_registry.get_name(ability_def.activator.activator_type) else {
        return;
    };

    match activator_name {
        "on_input" => {
            commands.entity(entity)
                .entry::<OnInputActivations>()
                .or_default()
                .and_modify(move |mut a| a.add(ability_id));
        }
        "passive" => {
            commands.entity(entity)
                .entry::<PassiveActivations>()
                .or_default()
                .and_modify(move |mut a| a.add(ability_id));
        }
        "while_held" => {
            let cooldown = extract_float_param(&ability_def.activator.params, "cooldown")
                .unwrap_or(0.05);
            commands.entity(entity)
                .entry::<WhileHeldActivations>()
                .or_default()
                .and_modify(move |mut a| a.add(ability_id, cooldown));
        }
        "interval" => {
            let interval = extract_float_param(&ability_def.activator.params, "interval")
                .unwrap_or(1.0);
            commands.entity(entity)
                .entry::<IntervalActivations>()
                .or_default()
                .and_modify(move |mut a| a.add(ability_id, interval));
        }
        _ => warn!("Unknown activator type: {}", activator_name),
    }
}

fn extract_float_param(
    params: &std::collections::HashMap<super::ids::ParamId, ParamValue>,
    _name: &str,
) -> Option<f32> {
    params.values().find_map(|v| match v {
        ParamValue::Float(f) => Some(*f),
        _ => None,
    })
}
