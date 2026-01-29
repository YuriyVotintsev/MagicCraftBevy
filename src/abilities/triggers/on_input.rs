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
pub struct OnInputTriggers {
    pub entries: Vec<OnInputEntry>,
}

pub struct OnInputEntry {
    pub ability_id: AbilityId,
}

impl OnInputTriggers {
    pub fn add(&mut self, ability_id: AbilityId) {
        self.entries.push(OnInputEntry { ability_id });
    }
}

pub fn on_input_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &OnInputTriggers,
        &AbilityInputs,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    for (entity, triggers, inputs, stats, transform, faction) in &query {
        for entry in &triggers.entries {
            let Some(input) = inputs.get(entry.ability_id) else {
                continue;
            };

            if !input.just_pressed {
                continue;
            }

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
pub struct OnInputHandler;

impl TriggerHandler for OnInputHandler {
    fn name(&self) -> &'static str {
        "on_input"
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &TriggerRegistry,
    ) {
        commands
            .entity(entity)
            .entry::<OnInputTriggers>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id));
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            on_input_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_trigger!(OnInputHandler);
