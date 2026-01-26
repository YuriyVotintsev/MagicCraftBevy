use bevy::prelude::*;
use std::collections::HashMap;

use crate::stats::StatRegistry;
use super::registry::{AbilityRegistry, ActivatorRegistry, EffectRegistry};
use super::ability_def::AbilityDef;
use super::activator_def::ActivatorDef;
use super::effect_def::{EffectDef, ParamValue};
use super::ids::AbilityId;

pub fn register_abilities(
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) {
    register_fireball(stat_registry, ability_registry, activator_registry, effect_registry);
    register_archer_shot(stat_registry, ability_registry, activator_registry, effect_registry);
}

fn register_fireball(
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> AbilityId {
    let fireball_id = ability_registry.allocate_id("fireball");
    let activator_type = activator_registry.get_id("on_input").unwrap();
    let spawn_projectile_type = effect_registry.get_id("spawn_projectile").unwrap();
    let damage_type = effect_registry.get_id("damage").unwrap();

    let amount_param = effect_registry.get_or_insert_param_id("amount");
    let on_hit_param = effect_registry.get_or_insert_param_id("on_hit");
    let physical_damage_id = stat_registry.get("physical_damage").unwrap();

    let damage_effect = EffectDef {
        effect_type: damage_type,
        params: HashMap::from([(amount_param, ParamValue::Stat(physical_damage_id))]),
    };

    let spawn_effect = EffectDef {
        effect_type: spawn_projectile_type,
        params: HashMap::from([(on_hit_param, ParamValue::EffectList(vec![damage_effect]))]),
    };

    let ability_def = AbilityDef {
        id: fireball_id,
        tags: Vec::new(),
        activator: ActivatorDef {
            activator_type,
            params: HashMap::new(),
        },
        effects: vec![spawn_effect],
    };

    ability_registry.register(ability_def);
    fireball_id
}

fn register_archer_shot(
    stat_registry: &StatRegistry,
    ability_registry: &mut AbilityRegistry,
    activator_registry: &ActivatorRegistry,
    effect_registry: &mut EffectRegistry,
) -> AbilityId {
    let archer_shot_id = ability_registry.allocate_id("archer_shot");
    let activator_type = activator_registry.get_id("on_input").unwrap();
    let spawn_projectile_type = effect_registry.get_id("spawn_projectile").unwrap();
    let damage_type = effect_registry.get_id("damage").unwrap();

    let amount_param = effect_registry.get_or_insert_param_id("amount");
    let on_hit_param = effect_registry.get_or_insert_param_id("on_hit");
    let speed_param = effect_registry.get_or_insert_param_id("speed");
    let size_param = effect_registry.get_or_insert_param_id("size");
    let physical_damage_id = stat_registry.get("physical_damage").unwrap();

    let damage_effect = EffectDef {
        effect_type: damage_type,
        params: HashMap::from([(amount_param, ParamValue::Stat(physical_damage_id))]),
    };

    let spawn_effect = EffectDef {
        effect_type: spawn_projectile_type,
        params: HashMap::from([
            (on_hit_param, ParamValue::EffectList(vec![damage_effect])),
            (speed_param, ParamValue::Float(400.0)),
            (size_param, ParamValue::Float(8.0)),
        ]),
    };

    let ability_def = AbilityDef {
        id: archer_shot_id,
        tags: Vec::new(),
        activator: ActivatorDef {
            activator_type,
            params: HashMap::new(),
        },
        effects: vec![spawn_effect],
    };

    ability_registry.register(ability_def);
    archer_shot_id
}
