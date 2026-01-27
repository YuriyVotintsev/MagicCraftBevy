use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

use crate::abilities::registry::{EffectExecutor, EffectRegistry};
use crate::abilities::effect_def::{EffectDef, ParamValue};
use crate::abilities::context::AbilityContext;

use super::spawn_projectile::{Projectile, Pierce};

const DEFAULT_ORB_COUNT: i32 = 3;
const DEFAULT_ORB_RADIUS: f32 = 80.0;
const DEFAULT_ORB_ANGULAR_SPEED: f32 = 3.0;
const DEFAULT_ORB_SIZE: f32 = 20.0;

#[derive(Component)]
pub struct OrbitingMovement {
    pub owner: Entity,
    pub radius: f32,
    pub angular_speed: f32,
    pub current_angle: f32,
}

pub struct SpawnOrbitingEffect;

impl EffectExecutor for SpawnOrbitingEffect {
    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let count = match def.get_param("count", registry) {
            Some(ParamValue::Int(v)) => *v,
            _ => DEFAULT_ORB_COUNT,
        };

        let radius = match def.get_param("radius", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_ORB_RADIUS,
        };

        let angular_speed = match def.get_param("angular_speed", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_ORB_ANGULAR_SPEED,
        };

        let size = match def.get_param("size", registry) {
            Some(ParamValue::Float(v)) => *v,
            _ => DEFAULT_ORB_SIZE,
        };

        let on_hit_effects = match def.get_param("on_hit", registry) {
            Some(ParamValue::EffectList(effects)) => effects.clone(),
            _ => Vec::new(),
        };

        for i in 0..count {
            let angle = 2.0 * PI * (i as f32) / (count as f32);
            let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
            let position = ctx.caster_position + offset.extend(0.0);

            commands.spawn((
                Name::new("Orb"),
                Projectile {
                    on_hit_effects: on_hit_effects.clone(),
                    context: ctx.clone(),
                },
                Pierce::Infinite,
                OrbitingMovement {
                    owner: ctx.caster,
                    radius,
                    angular_speed,
                    current_angle: angle,
                },
                ctx.caster_faction,
                Collider::circle(size / 2.0),
                Sensor,
                CollisionEventsEnabled,
                RigidBody::Kinematic,
                Sprite {
                    color: Color::srgb(0.3, 0.7, 1.0),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(position),
            ));
        }
    }
}
