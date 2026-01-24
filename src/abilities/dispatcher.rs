use bevy::prelude::*;

use crate::stats::ComputedStats;
use super::ids::AbilityId;
use super::context::AbilityContext;
use super::components::{Abilities, AbilityInput};
use super::registry::{ActivatorRegistry, EffectRegistry, AbilityRegistry};
use super::activator_def::ActivationResult;

pub fn ability_dispatcher(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Abilities,
        &mut AbilityInput,
        &ComputedStats,
        &Transform,
    )>,
    ability_registry: Res<AbilityRegistry>,
    activator_registry: Res<ActivatorRegistry>,
    effect_registry: Res<EffectRegistry>,
) {
    for (entity, mut abilities, mut input, stats, transform) in &mut query {
        let wanted_ability = input.want_to_cast;

        for ability_instance in abilities.list.iter_mut() {
            let Some(ability_def) = ability_registry.get(ability_instance.def_id) else {
                continue;
            };

            let mut ctx = AbilityContext::new(
                entity,
                stats,
                transform.translation,
                ability_instance.def_id,
            ).with_tags(ability_def.tags.clone());

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
            );

            if result == ActivationResult::Ready {
                for effect_def in &ability_def.effects {
                    effect_registry.execute(effect_def, &ctx, &mut commands);
                }
            }
        }

        input.clear();
    }
}
