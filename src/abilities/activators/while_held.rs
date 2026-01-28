use bevy::prelude::*;

use crate::abilities::{AbilityId, AbilityInput, AbilityRegistry, EffectRegistry, AbilityContext};
use crate::stats::ComputedStats;
use crate::Faction;

#[derive(Component, Default)]
pub struct WhileHeldActivations {
    pub entries: Vec<WhileHeldEntry>,
}

pub struct WhileHeldEntry {
    pub ability_id: AbilityId,
    pub cooldown: f32,
    pub timer: f32,
}

impl WhileHeldActivations {
    pub fn add(&mut self, ability_id: AbilityId, cooldown: f32) {
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
        &mut WhileHeldActivations,
        &AbilityInput,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut activations, input, stats, transform, faction) in &mut query {
        let held = input.holding.as_ref();

        for entry in &mut activations.entries {
            entry.timer = (entry.timer - delta).max(0.0);

            let Some(held) = held else {
                continue;
            };

            if held.ability_id != entry.ability_id {
                continue;
            }

            if entry.timer > 0.0 {
                continue;
            }

            entry.timer = entry.cooldown;

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
            .with_target_direction(held.target_direction)
            .with_target_point(held.target_point);

            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }
        }
    }
}
