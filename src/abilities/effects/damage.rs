use bevy::prelude::*;

use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::abilities::effect_def::EffectDef;
use crate::abilities::context::AbilityContext;
use crate::stats::PendingDamage;

#[derive(Default)]
pub struct DamageHandler;

impl EffectHandler for DamageHandler {
    fn name(&self) -> &'static str {
        "damage"
    }

    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let Some(amount) = def.get_f32("amount", &ctx.stats_snapshot, registry) else {
            return;
        };

        let Some(target) = ctx.get_param_entity("target") else {
            return;
        };

        if let Ok(mut entity_commands) = commands.get_entity(target) {
            entity_commands.insert(PendingDamage(amount));
        }
    }
}

register_effect!(DamageHandler);
