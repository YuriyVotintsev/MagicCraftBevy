use std::collections::HashMap;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use crate::register_node;
use rand::Rng;

use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::abilities::{ParamValue, ParamValueRaw, ParseNodeParams, resolve_param_value};
use crate::abilities::node::{NodeHandler, NodeKind};
use crate::abilities::context::Target;
use crate::abilities::events::ExecuteNodeEvent;
use crate::abilities::AbilitySource;
use crate::building_blocks::triggers::on_collision::OnCollisionTrigger;
use crate::physics::{GameLayer, Wall};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS, StatRegistry};
use crate::Faction;
use crate::{Growing, Lifetime};
use crate::GameState;

#[derive(Debug, Clone)]
pub struct ProjectileParams {
    pub speed: ParamValue,
    pub size: ParamValue,
    pub spread: ParamValue,
    pub lifetime: Option<ParamValue>,
    pub start_size: Option<ParamValue>,
    pub end_size: Option<ParamValue>,
    pub pierce: Option<ParamValue>,
}

impl ParseNodeParams for ProjectileParams {
    fn parse(raw: &HashMap<String, ParamValueRaw>, stat_registry: &StatRegistry) -> Self {
        Self {
            speed: raw.get("speed").map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(800.0)),
            size: raw.get("size").map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(15.0)),
            spread: raw.get("spread").map(|v| resolve_param_value(v, stat_registry))
                .unwrap_or(ParamValue::Float(0.0)),
            lifetime: raw.get("lifetime").map(|v| resolve_param_value(v, stat_registry)),
            start_size: raw.get("start_size").map(|v| resolve_param_value(v, stat_registry)),
            end_size: raw.get("end_size").map(|v| resolve_param_value(v, stat_registry)),
            pierce: raw.get("pierce").map(|v| resolve_param_value(v, stat_registry)),
        }
    }
}

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub enum Pierce {
    Count(u32),
    Infinite,
}

fn execute_spawn_projectile_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteNodeEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    let Some(handler_id) = node_registry.get_id("spawn_projectile") else {
        return;
    };

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

        let params = node_def.params.expect_action().expect_spawn_projectile();

        let caster_stats = stats_query
            .get(event.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let speed = params.speed.evaluate_f32(&caster_stats);
        let size = params.size.evaluate_f32(&caster_stats);
        let spread = params.spread.evaluate_f32(&caster_stats);
        let lifetime = params.lifetime.as_ref().map(|p| p.evaluate_f32(&caster_stats));
        let start_size = params.start_size.as_ref().map(|p| p.evaluate_f32(&caster_stats));
        let end_size = params.end_size.as_ref().map(|p| p.evaluate_f32(&caster_stats));
        let pierce = params.pierce.as_ref().map(|p| Pierce::Count(p.evaluate_i32(&caster_stats) as u32));

        let base_direction = match event.context.target {
            Some(Target::Direction(d)) => d.truncate().normalize_or_zero(),
            _ => Vec2::X,
        };

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
        let initial_size = start_size.unwrap_or(size);

        let projectile_layers = match event.context.caster_faction {
            Faction::Player => CollisionLayers::new(
                GameLayer::PlayerProjectile,
                [GameLayer::Enemy, GameLayer::Wall],
            ),
            Faction::Enemy => CollisionLayers::new(
                GameLayer::EnemyProjectile,
                [GameLayer::Player, GameLayer::Wall],
            ),
        };

        let mut entity_commands = commands.spawn((
            Name::new("Projectile"),
            Projectile,
            AbilitySource::new(
                event.ability_id,
                event.node_id,
                event.context.caster,
                event.context.caster_faction,
            ),
            event.context.caster_faction,
            Collider::circle(initial_size / 2.0),
            Sensor,
            CollisionEventsEnabled,
            RigidBody::Kinematic,
            LinearVelocity(velocity),
            projectile_layers,
            Sprite {
                color: Color::srgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::splat(initial_size)),
                ..default()
            },
            Transform::from_translation(event.context.source.as_point().unwrap_or(Vec3::ZERO)),
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

        if let Some(trigger) = OnCollisionTrigger::if_configured(ability_def, event.node_id, &node_registry) {
            entity_commands.insert(trigger);
        }
    }
}

#[derive(Default)]
pub struct SpawnProjectileHandler;

impl NodeHandler for SpawnProjectileHandler {
    fn name(&self) -> &'static str {
        "spawn_projectile"
    }

    fn kind(&self) -> NodeKind {
        NodeKind::Action
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_spawn_projectile_action
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            projectile_collision_physics.in_set(GameSet::AbilityExecution),
        );
    }
}

fn projectile_collision_physics(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    projectile_query: Query<(&Projectile, &Faction), Without<Wall>>,
    mut pierce_query: Query<&mut Pierce>,
    target_query: Query<&Faction, Without<Projectile>>,
    wall_query: Query<(), With<Wall>>,
) {
    let mut despawned: HashSet<Entity> = HashSet::default();

    for event in collision_events.read() {
        let entity1 = event.collider1;
        let entity2 = event.collider2;

        let (projectile_entity, other_entity) =
            if projectile_query.contains(entity1) {
                (entity1, entity2)
            } else if projectile_query.contains(entity2) {
                (entity2, entity1)
            } else {
                continue;
            };

        if despawned.contains(&projectile_entity) {
            continue;
        }

        if wall_query.contains(other_entity) {
            let has_pierce_infinite = pierce_query
                .get(projectile_entity)
                .map(|p| matches!(*p, Pierce::Infinite))
                .unwrap_or(false);

            if !has_pierce_infinite {
                if let Ok(mut entity_commands) = commands.get_entity(projectile_entity) {
                    entity_commands.despawn();
                    despawned.insert(projectile_entity);
                }
            }
            continue;
        }

        if projectile_query.contains(other_entity) {
            continue;
        }

        let Ok((_projectile, proj_faction)) = projectile_query.get(projectile_entity) else {
            continue;
        };
        let Ok(target_faction) = target_query.get(other_entity) else {
            continue;
        };

        if proj_faction == target_faction {
            continue;
        }

        let should_despawn = match pierce_query.get_mut(projectile_entity) {
            Err(_) => true,
            Ok(pierce) => match pierce.into_inner() {
                Pierce::Infinite => false,
                Pierce::Count(n) => {
                    if *n <= 1 {
                        true
                    } else {
                        *n -= 1;
                        false
                    }
                }
            },
        };

        if should_despawn {
            if let Ok(mut entity_commands) = commands.get_entity(projectile_entity) {
                entity_commands.despawn();
                despawned.insert(projectile_entity);
            }
        }
    }
}

register_node!(SpawnProjectileHandler, params: ProjectileParams, name: "spawn_projectile");
