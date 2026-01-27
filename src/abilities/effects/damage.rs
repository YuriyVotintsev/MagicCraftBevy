use bevy::prelude::*;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;
use crate::stats::PendingDamage;

pub struct DamageEffect;

impl EffectExecutor for DamageEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let amount = match def.get_param("amount", registry) {
            Some(ParamValue::Float(v)) => *v,
            Some(ParamValue::Stat(stat_id)) => ctx.stats_snapshot.get(*stat_id),
            Some(ParamValue::Expr(expr)) => expr.evaluate_computed(&ctx.stats_snapshot),
            _ => return,
        };

        let Some(target) = ctx.get_param_entity("target") else {
            return;
        };

        if let Ok(mut entity_commands) = commands.get_entity(target) {
            entity_commands.insert(PendingDamage(amount));
        }
    }
}
