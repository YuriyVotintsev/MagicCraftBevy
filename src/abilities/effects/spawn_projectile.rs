use bevy::prelude::*;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;
use crate::abilities::owner::OwnedBy;
use crate::stats::StatId;

#[derive(Component)]
pub struct Projectile {
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

#[derive(Component)]
pub struct ProjectileVelocity(pub Vec2);

pub struct SpawnProjectileEffect;

impl EffectExecutor for SpawnProjectileEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let speed = match def.get_param("speed", registry) {
            Some(ParamValue::Float(v)) => *v,
            Some(ParamValue::Expr(expr)) => expr.evaluate(&ctx.stats_snapshot),
            _ => ctx.stats_snapshot.get(StatId::ProjectileSpeed),
        };

        let on_hit_effects = match def.get_param("on_hit", registry) {
            Some(ParamValue::EffectList(effects)) => effects.clone(),
            _ => Vec::new(),
        };

        let direction = ctx.target_direction.unwrap_or(Vec3::X).truncate().normalize_or_zero();
        let velocity = direction * speed;

        commands.spawn((
            Projectile {
                on_hit_effects,
                context: ctx.clone(),
            },
            ProjectileVelocity(velocity),
            OwnedBy::from_arc(ctx.caster, ctx.stats_snapshot.clone()),
            Sprite {
                color: Color::srgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::splat(15.0)),
                ..default()
            },
            Transform::from_translation(ctx.caster_position),
        ));
    }
}
