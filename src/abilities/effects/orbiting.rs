use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::abilities::owner::OwnedBy;
use crate::abilities::events::ExecuteEffectEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
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

fn execute_orbiting_effect(
    mut commands: Commands,
    mut effect_events: MessageReader<ExecuteEffectEvent>,
    effect_registry: Res<EffectRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in effect_events.read() {
        let Some(handler_id) = effect_registry.get_id("spawn_orbiting") else {
            continue;
        };
        if event.effect.effect_type != handler_id {
            continue;
        }

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();

        let count = event
            .effect
            .get_i32("count", &caster_stats, &effect_registry)
            .unwrap_or(DEFAULT_ORB_COUNT);
        let radius = event
            .effect
            .get_f32("radius", &caster_stats, &effect_registry)
            .unwrap_or(DEFAULT_ORB_RADIUS);
        let angular_speed = event
            .effect
            .get_f32("angular_speed", &caster_stats, &effect_registry)
            .unwrap_or(DEFAULT_ORB_ANGULAR_SPEED);
        let size = event
            .effect
            .get_f32("size", &caster_stats, &effect_registry)
            .unwrap_or(DEFAULT_ORB_SIZE);
        let on_hit_effects = event
            .effect
            .get_effect_list("on_hit", &effect_registry)
            .cloned()
            .unwrap_or_default();

        let orb_layers = match event.context.caster_faction {
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
            let position = event.context.caster_position + offset.extend(0.0);

            commands.spawn((
                Name::new("Orb"),
                Projectile {
                    on_hit_effects: on_hit_effects.clone(),
                    context: event.context.clone(),
                },
                Pierce::Infinite,
                OrbitingMovement {
                    owner: event.context.caster,
                    radius,
                    angular_speed,
                    current_angle: angle,
                },
                OwnedBy::new(event.context.caster),
                event.context.caster_faction,
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
}

#[derive(Default)]
pub struct SpawnOrbitingHandler;

impl EffectHandler for SpawnOrbitingHandler {
    fn name(&self) -> &'static str {
        "spawn_orbiting"
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_orbiting_effect
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
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
