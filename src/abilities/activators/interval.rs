use bevy::prelude::*;

use crate::abilities::{AbilityId, AbilityRegistry, EffectRegistry, AbilityContext};
use crate::stats::ComputedStats;
use crate::Faction;

#[derive(Component, Default)]
pub struct IntervalActivations {
    pub entries: Vec<IntervalEntry>,
}

pub struct IntervalEntry {
    pub ability_id: AbilityId,
    pub interval: f32,
    pub timer: f32,
}

impl IntervalActivations {
    pub fn add(&mut self, ability_id: AbilityId, interval: f32) {
        self.entries.push(IntervalEntry {
            ability_id,
            interval,
            timer: interval,
        });
    }
}

pub fn interval_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut IntervalActivations,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    let delta = time.delta_secs();

    for (entity, mut activations, stats, transform, faction) in &mut query {
        for entry in &mut activations.entries {
            entry.timer -= delta;

            if entry.timer > 0.0 {
                continue;
            }

            entry.timer = entry.interval;

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
        }
    }
}
