use bevy::prelude::*;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;
use crate::stats::StatId;

pub struct DamageEffect;

impl EffectExecutor for DamageEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        _commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let amount = match def.get_param("amount", registry) {
            Some(ParamValue::Float(v)) => *v,
            Some(ParamValue::Expr(expr)) => expr.evaluate(&ctx.stats_snapshot),
            _ => ctx.stats_snapshot.get(StatId::PhysicalDamage),
        };

        let target = ctx.get_param_entity("target");

        info!(
            "Damage effect: {} damage from {:?} to {:?}",
            amount, ctx.caster, target
        );
    }
}
