use std::collections::HashMap;
use bevy::prelude::*;

use crate::abilities::ids::ParamId;
use crate::abilities::effect_def::ParamValue;
use crate::abilities::registry::ActivatorHandler;
use crate::abilities::{AbilityId, AbilityRegistry, ActivatorRegistry, EffectRegistry, AbilityContext};
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::GameState;

#[derive(Component, Default)]
pub struct PassiveActivations {
    pub entries: Vec<PassiveEntry>,
}

pub struct PassiveEntry {
    pub ability_id: AbilityId,
    pub activated: bool,
}

impl PassiveActivations {
    pub fn add(&mut self, ability_id: AbilityId) {
        self.entries.push(PassiveEntry {
            ability_id,
            activated: false,
        });
    }
}

pub fn passive_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut PassiveActivations,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    for (entity, mut activations, stats, transform, faction) in &mut query {
        for entry in &mut activations.entries {
            if entry.activated {
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
            );

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }

            entry.activated = true;
        }
    }
}

#[derive(Default)]
pub struct PassiveHandler;

impl ActivatorHandler for PassiveHandler {
    fn name(&self) -> &'static str {
        "passive"
    }

    fn add_to_entity(
        &self,
        commands: &mut Commands,
        entity: Entity,
        ability_id: AbilityId,
        _params: &HashMap<ParamId, ParamValue>,
        _registry: &ActivatorRegistry,
    ) {
        commands
            .entity(entity)
            .entry::<PassiveActivations>()
            .or_default()
            .and_modify(move |mut a| a.add(ability_id));
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            passive_system
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

register_activator!(PassiveHandler);
