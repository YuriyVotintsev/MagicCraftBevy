use std::collections::HashMap;
use bevy::prelude::*;

use crate::abilities::ids::ParamId;
use crate::abilities::effect_def::ParamValue;
use crate::abilities::registry::TriggerHandler;
use crate::abilities::{AbilityId, AbilityInputs, AbilityRegistry, TriggerRegistry, EffectRegistry, AbilityContext};
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct WhileHeldTriggers {
    pub entries: Vec<WhileHeldEntry>,
}

pub struct WhileHeldEntry {
    pub ability_id: AbilityId,
    pub cooldown: ParamValue,
    pub timer: f32,
}

impl WhileHeldTriggers {
    pub fn add(&mut self, ability_id: AbilityId, cooldown: ParamValue) {
        self.entries.push(WhileHeldEntry {
            ability_id,
            cooldown,
            timer: 0.0,
        });
    }
}

pub fn while_held_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut WhileHeldTriggers,
        &AbilityInputs,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut triggers, inputs, stats, transform, faction) in &mut query {
        for entry in &mut triggers.entries {
            entry.timer = (entry.timer - delta).max(0.0);

            let Some(input) = inputs.get(entry.ability_id) else {
                continue;
            };

            if !input.pressed {
                continue;
            }

            if entry.timer > 0.0 {
                continue;
            }

            let cooldown = entry.cooldown.evaluate_f32(stats).unwrap_or(0.05);
            entry.timer = cooldown;

            let Some(ability_def) = ability_registry.get(entry.ability_id) else {
                continue;
            };

            let ctx = AbilityContext::new(
                entity,
                *faction,
                stats,
                transform.translation,
            )
            .with_target_direction(input.direction)
            .with_target_point(input.point);

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }
        }
    }
}

#[derive(Default)]
pub struct WhileHeldHandler;

impl TriggerHandler for WhileHeldHandler {
    fn name(&self) -> &'static str {
        "while_held"
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        params: &HashMap<ParamId, ParamValue>,
        registry: &TriggerRegistry,
    ) {
        let cooldown = registry
            .get_param_id("cooldown")
            .and_then(|id| params.get(&id).cloned())
            .unwrap_or(ParamValue::Float(0.05));
        commands
            .entity(entity)
            .entry::<WhileHeldTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id, cooldown));
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            while_held_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_trigger!(WhileHeldHandler);
