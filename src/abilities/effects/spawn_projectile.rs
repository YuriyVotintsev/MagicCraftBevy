use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;
use crate::abilities::owner::OwnedBy;

const DEFAULT_PROJECTILE_SPEED: f32 = 800.0;
const DEFAULT_PROJECTILE_SIZE: f32 = 15.0;

#[derive(Component)]
pub struct Projectile {
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

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
            Some(ParamValue::Stat(stat_id)) => ctx.stats_snapshot.get(*stat_id),
            Some(ParamValue::Expr(expr)) => expr.evaluate_computed(&ctx.stats_snapshot),
            _ => DEFAULT_PROJECTILE_SPEED,
        };

        let size = match def.get_param("size", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_PROJECTILE_SIZE,
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
            Collider::circle(size / 2.0),
            Sensor,
            CollisionEventsEnabled,
            RigidBody::Kinematic,
            LinearVelocity(velocity),
            OwnedBy::from_arc(ctx.caster, ctx.stats_snapshot.clone()),
            Sprite {
                color: Color::srgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(ctx.caster_position),
        ));
    }
}
