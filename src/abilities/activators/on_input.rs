use bevy::prelude::*;

use crate::abilities::{AbilityId, AbilityInputs, AbilityRegistry, EffectRegistry, AbilityContext};
use crate::stats::ComputedStats;
use crate::Faction;

#[derive(Component, Default)]
pub struct OnInputActivations {
    pub entries: Vec<OnInputEntry>,
}

pub struct OnInputEntry {
    pub ability_id: AbilityId,
}

impl OnInputActivations {
    pub fn add(&mut self, ability_id: AbilityId) {
        self.entries.push(OnInputEntry { ability_id });
    }
}

pub fn on_input_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &OnInputActivations,
        &AbilityInputs,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    for (entity, activations, inputs, stats, transform, faction) in &query {
        for entry in &activations.entries {
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
                entry.ability_id,
            )
            .with_target_direction(input.direction)
            .with_target_point(input.point);

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }
        }
    }
}
