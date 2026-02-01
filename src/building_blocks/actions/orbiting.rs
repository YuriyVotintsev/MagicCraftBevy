use std::f32::consts::PI;
use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::GenerateRaw;

use crate::register_node;
use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::ParamValue;
use crate::abilities::ids::NodeTypeId;
use crate::abilities::AbilitySource;
use crate::common::AttachedTo;
use crate::building_blocks::actions::ActionParams;
use crate::building_blocks::triggers::on_collision::OnCollisionTrigger;
use crate::abilities::events::ExecuteNodeEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::Faction;
use crate::GameState;

use super::spawn_projectile::Pierce;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Action)]
pub struct OrbitingParams {
    #[raw(default = 3.0)]
    pub count: ParamValue,
    #[raw(default = 80.0)]
    pub radius: ParamValue,
    #[raw(default = 3.0)]
    pub angular_speed: ParamValue,
    #[raw(default = 20.0)]
    pub size: ParamValue,
}

#[derive(Component)]
pub struct OrbitingMovement {
    pub radius: f32,
    pub angular_speed: f32,
    pub current_angle: f32,
}

fn execute_orbiting_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteNodeEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
    mut cached_id: Local<Option<NodeTypeId>>,
) {
    let handler_id = *cached_id.get_or_insert_with(|| {
        node_registry.get_id("OrbitingParams")
            .expect("OrbitingParams not registered")
    });

    for event in action_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };
        let Some(node_def) = ability_def.get_node(event.node_id) else {
            continue;
        };

        if node_def.node_type != handler_id {
            continue;
        }

        let ActionParams::OrbitingParams(params) = node_def.params.unwrap_action() else {
            continue;
        };

        let caster_stats = stats_query
            .get(event.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let count = params.count.evaluate_i32(&caster_stats);
        let radius = params.radius.evaluate_f32(&caster_stats);
        let angular_speed = params.angular_speed.evaluate_f32(&caster_stats);
        let size = params.size.evaluate_f32(&caster_stats);

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
            let source_point = event.context.source.as_point().unwrap_or(Vec3::ZERO);
            let position = source_point + offset.extend(0.0);

            let mut entity = commands.spawn((
                Name::new("Orb"),
                AbilitySource::new(
                    event.ability_id,
                    event.node_id,
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

            if let Some(trigger) = OnCollisionTrigger::if_configured(ability_def, event.node_id, &node_registry) {
                entity.insert(trigger);
            }
        }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        execute_orbiting_action
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        update_orbiting_positions.in_set(GameSet::AbilityExecution),
    );
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

register_node!(OrbitingParams);
