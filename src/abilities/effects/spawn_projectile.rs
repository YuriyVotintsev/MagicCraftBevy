use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;
use crate::abilities::owner::OwnedBy;
use crate::{Growing, Lifetime};

const DEFAULT_PROJECTILE_SPEED: f32 = 800.0;
const DEFAULT_PROJECTILE_SIZE: f32 = 15.0;

#[derive(Component)]
pub struct Projectile {
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

#[derive(Component)]
pub enum Pierce {
    Count(u32),
    Infinite,
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

        let spread = match def.get_param("spread", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => 0.0,
        };

        let lifetime = match def.get_param("lifetime", registry) {
            Some(ParamValue::Float(v)) => Some(*v),
            _ => None,
        };

        let start_size = match def.get_param("start_size", registry) {
            Some(ParamValue::Float(v)) => Some(*v),
            _ => None,
        };

        let end_size = match def.get_param("end_size", registry) {
            Some(ParamValue::Float(v)) => Some(*v),
            _ => None,
        };

        let base_direction = ctx.target_direction.unwrap_or(Vec3::X).truncate().normalize_or_zero();
        let direction = if spread > 0.0 {
            let spread_rad = spread.to_radians();
            let angle_offset = rand::rng().random_range(-spread_rad..spread_rad);
            let cos = angle_offset.cos();
            let sin = angle_offset.sin();
            Vec2::new(
                base_direction.x * cos - base_direction.y * sin,
                base_direction.x * sin + base_direction.y * cos,
            )
        } else {
            base_direction
        };
        let velocity = direction * speed;

        let pierce = match def.get_param("pierce", registry) {
            Some(ParamValue::Int(n)) => Some(Pierce::Count(*n as u32)),
            _ => None,
        };

        let initial_size = start_size.unwrap_or(size);

        let mut entity_commands = commands.spawn((
            Name::new("Projectile"),
            Projectile {
                on_hit_effects,
                context: ctx.clone(),
            },
            ctx.caster_faction,
            Collider::circle(initial_size / 2.0),
            Sensor,
            CollisionEventsEnabled,
            RigidBody::Kinematic,
            LinearVelocity(velocity),
            OwnedBy::from_arc(ctx.caster, ctx.stats_snapshot.clone()),
            Sprite {
                color: Color::srgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::splat(initial_size)),
                ..default()
            },
            Transform::from_translation(ctx.caster_position),
        ));

        if let Some(pierce) = pierce {
            entity_commands.insert(pierce);
        }

        if let Some(lt) = lifetime {
            entity_commands.insert(Lifetime { remaining: lt });

            if let (Some(ss), Some(es)) = (start_size, end_size) {
                entity_commands.insert(Growing {
                    start_size: ss,
                    end_size: es,
                    duration: lt,
                    elapsed: 0.0,
                });
            }
        }
    }
}
