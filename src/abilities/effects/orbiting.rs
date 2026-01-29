use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::abilities::effect_def::EffectDef;
use crate::abilities::context::AbilityContext;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

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

#[derive(Default)]
pub struct SpawnOrbitingHandler;

impl EffectHandler for SpawnOrbitingHandler {
    fn name(&self) -> &'static str {
        "spawn_orbiting"
    }

    fn execute(
        &self,
        def: &EffectDef,
        ctx: &AbilityContext,
        commands: &mut Commands,
        registry: &EffectRegistry,
    ) {
        let stats = &ctx.stats_snapshot;
        let count = def.get_i32("count", stats, registry).unwrap_or(DEFAULT_ORB_COUNT);
        let radius = def.get_f32("radius", stats, registry).unwrap_or(DEFAULT_ORB_RADIUS);
        let angular_speed = def.get_f32("angular_speed", stats, registry).unwrap_or(DEFAULT_ORB_ANGULAR_SPEED);
        let size = def.get_f32("size", stats, registry).unwrap_or(DEFAULT_ORB_SIZE);
        let on_hit_effects = def.get_effect_list("on_hit", registry).cloned().unwrap_or_default();

        let orb_layers = match ctx.caster_faction {
            Faction::Player => CollisionLayers::new(
                GameLayer::PlayerProjectile,
                [GameLayer::Enemy, GameLayer::Wall],
            ),
            Faction::Enemy => CollisionLayers::new(
                GameLayer::EnemyProjectile,
                [GameLayer::Player, GameLayer::Wall],
            ),
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
                orb_layers,
                Sprite {
                    color: Color::srgb(0.3, 0.7, 1.0),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(position),
            ));
        }
    }

    fn register_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_orbiting_positions.in_set(GameSet::AbilityExecution),
        )
        .add_systems(
            PostUpdate,
            cleanup_orbiting_on_owner_despawn.run_if(in_state(GameState::Playing)),
        );
    }
}

fn update_orbiting_positions(
    time: Res<Time>,
    owner_query: Query<&Transform, Without<OrbitingMovement>>,
    mut orb_query: Query<(&mut OrbitingMovement, &mut Transform)>,
) {
    for (mut orbiting, mut transform) in &mut orb_query {
        orbiting.current_angle += orbiting.angular_speed * time.delta_secs();

        if let Ok(owner_transform) = owner_query.get(orbiting.owner) {
            let offset = Vec2::new(
                orbiting.current_angle.cos() * orbiting.radius,
                orbiting.current_angle.sin() * orbiting.radius,
            );
            transform.translation = owner_transform.translation + offset.extend(0.0);
        }
    }
}

fn cleanup_orbiting_on_owner_despawn(
    mut commands: Commands,
    orb_query: Query<(Entity, &OrbitingMovement)>,
    owner_query: Query<&Transform>,
) {
    for (entity, orbiting) in &orb_query {
        if owner_query.get(orbiting.owner).is_err() {
            commands.entity(entity).despawn();
        }
    }
}

register_effect!(SpawnOrbitingHandler);
