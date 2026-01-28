use bevy::prelude::*;

use crate::Faction;
use crate::stats::ComputedStats;
use super::context::AbilityContext;
use super::components::{Abilities, AbilityInput};
use super::registry::{ActivatorRegistry, EffectRegistry, AbilityRegistry};
use super::activator_def::ActivationResult;

pub fn ability_dispatcher(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Abilities,
        &mut AbilityInput,
        &ComputedStats,
        &Transform,
        &Faction,
    )>,
    ability_registry: Res<AbilityRegistry>,
    activator_registry: Res<ActivatorRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    let delta_time = time.delta_secs();
    for (entity, mut abilities, mut input, stats, transform, faction) in &mut query {
        let Some(wanted_ability_id) = input.want_to_cast else {
            continue;
        };

        let Some(ability_instance) = abilities.get_mut(wanted_ability_id) else {
            input.clear();
            continue;
        };

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

        let result = activator_registry.check(
            &ability_def.activator,
            &mut ability_instance.state,
            &mut ctx,
            &input,
            delta_time,
        );

        if result == ActivationResult::Ready {
            for effect_def in &ability_def.effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }
        }

        input.clear();
    }
}
