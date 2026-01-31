use std::f32::consts::PI;
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::registry::{ActionHandler, ActionRegistry};
use crate::abilities::AttachedTo;
use crate::abilities::AbilitySource;
use crate::abilities::events::ExecuteActionEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::GameState;

use super::spawn_projectile::Pierce;

const DEFAULT_ORB_COUNT: i32 = 3;
const DEFAULT_ORB_RADIUS: f32 = 80.0;
const DEFAULT_ORB_ANGULAR_SPEED: f32 = 3.0;
const DEFAULT_ORB_SIZE: f32 = 20.0;

#[derive(Component)]
pub struct OrbitingMovement {
    pub radius: f32,
    pub angular_speed: f32,
    pub current_angle: f32,
}

fn execute_orbiting_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteActionEvent>,
    action_registry: Res<ActionRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in action_events.read() {
        let Some(handler_id) = action_registry.get_id("spawn_orbiting") else {
            continue;
        };
        if event.action.action_type != handler_id {
            continue;
        }

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();

        let count = event
            .action
            .get_i32("count", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_ORB_COUNT);
        let radius = event
            .action
            .get_f32("radius", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_ORB_RADIUS);
        let angular_speed = event
            .action
            .get_f32("angular_speed", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_ORB_ANGULAR_SPEED);
        let size = event
            .action
            .get_f32("size", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_ORB_SIZE);

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
            let position = event.context.source_point + offset.extend(0.0);

            commands.spawn((
                Name::new("Orb"),
                AbilitySource::new(
                    event.action.clone(),
                    event.context.caster,
                    event.context.caster_faction,
                ),
                Pierce::Infinite,
                OrbitingMovement {
                    radius,
                    angular_speed,
                    current_angle: angle,
                },
                AttachedTo { owner: event.context.caster },
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

impl ActionHandler for SpawnOrbitingHandler {
    fn name(&self) -> &'static str {
        "spawn_orbiting"
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_orbiting_action
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_orbiting_positions.in_set(GameSet::AbilityExecution),
        );
    }
}

fn update_orbiting_positions(
    time: Res<Time>,
    owner_query: Query<&Transform, Without<OrbitingMovement>>,
    mut orb_query: Query<(&AttachedTo, &mut OrbitingMovement, &mut Transform)>,
) {
    for (attached, mut orbiting, mut transform) in &mut orb_query {
        orbiting.current_angle += orbiting.angular_speed * time.delta_secs();

        if let Ok(owner_transform) = owner_query.get(attached.owner) {
            let offset = Vec2::new(
                orbiting.current_angle.cos() * orbiting.radius,
                orbiting.current_angle.sin() * orbiting.radius,
            );
            transform.translation = owner_transform.translation + offset.extend(0.0);
        }
    }
}

register_action!(SpawnOrbitingHandler);
