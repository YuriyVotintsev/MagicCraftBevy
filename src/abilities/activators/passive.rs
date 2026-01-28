use bevy::prelude::*;

use crate::abilities::{AbilityId, AbilityRegistry, EffectRegistry, AbilityContext};
use crate::stats::ComputedStats;
use crate::Faction;

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
                entry.ability_id,
            );

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }

            entry.activated = true;
        }
    }
}
