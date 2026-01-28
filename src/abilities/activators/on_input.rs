use bevy::prelude::*;

use crate::abilities::{AbilityId, AbilityInput, AbilityRegistry, EffectRegistry, AbilityContext};
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
    mut query: Query<(
        Entity,
        &OnInputActivations,
        &mut AbilityInput,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    for (entity, activations, mut input, stats, transform, faction) in &mut query {
        let Some(wanted_ability_id) = input.want_to_cast else {
            continue;
        };

        let has_ability = activations.entries.iter().any(|e| e.ability_id == wanted_ability_id);
        if !has_ability {
            continue;
        }

        let Some(ability_def) = ability_registry.get(wanted_ability_id) else {
            input.clear();
            continue;
        };

        let mut ctx = AbilityContext::new(
            entity,
            *faction,
            stats,
            transform.translation,
            wanted_ability_id,
        );

        if let Some(dir) = input.target_direction {
            ctx = ctx.with_target_direction(dir);
        }
        if let Some(pt) = input.target_point {
            ctx = ctx.with_target_point(pt);
        }

        for effect_def in &ability_def.effects {
            effect_registry.execute(effect_def, &ctx, &mut commands);
        }

        input.clear();
    }
}
