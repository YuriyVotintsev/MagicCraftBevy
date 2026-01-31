use std::sync::Arc;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use rand::Rng;

use crate::abilities::registry::{ActionHandler, ActionRegistry};
use crate::abilities::trigger_def::ActionDef;
use crate::abilities::context::{AbilityContext, ContextValue};
use crate::abilities::events::ExecuteActionEvent;
use crate::abilities::AbilitySource;
use crate::abilities::ids::TriggerTypeId;
use crate::physics::{GameLayer, Wall};
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::{Growing, Lifetime};
use crate::GameState;

const DEFAULT_PROJECTILE_SPEED: f32 = 800.0;
const DEFAULT_PROJECTILE_SIZE: f32 = 15.0;

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
    pub size: f32,
}

#[derive(Component)]
pub enum Pierce {
    Count(u32),
    Infinite,
}

fn execute_spawn_projectile_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteActionEvent>,
    action_registry: Res<ActionRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in action_events.read() {
        let Some(handler_id) = action_registry.get_id("spawn_projectile") else {
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

        let speed = event
            .action
            .get_f32("speed", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_PROJECTILE_SPEED);
        let size = event
            .action
            .get_f32("size", &caster_stats, &action_registry)
            .unwrap_or(DEFAULT_PROJECTILE_SIZE);
        let spread = event
            .action
            .get_f32("spread", &caster_stats, &action_registry)
            .unwrap_or(0.0);
        let lifetime = event.action.get_f32("lifetime", &caster_stats, &action_registry);
        let start_size = event.action.get_f32("start_size", &caster_stats, &action_registry);
        let end_size = event.action.get_f32("end_size", &caster_stats, &action_registry);
        let pierce = event
            .action
            .get_i32("pierce", &caster_stats, &action_registry)
            .map(|n| Pierce::Count(n as u32));

        let base_direction = event
            .context
            .target_direction
            .unwrap_or(Vec3::X)
            .truncate()
            .normalize_or_zero();

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
            Projectile { speed, size },
            AbilitySource::new(
                event.action.clone(),
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
            Transform::from_translation(event.context.caster_position),
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

#[derive(Default)]
pub struct SpawnProjectileHandler;

impl ActionHandler for SpawnProjectileHandler {
    fn name(&self) -> &'static str {
        "spawn_projectile"
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
            projectile_collision.in_set(GameSet::AbilityExecution),
        );
    }
}

fn projectile_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    mut action_events: MessageWriter<ExecuteActionEvent>,
    projectile_query: Query<(&Projectile, &AbilitySource, &Faction)>,
    mut pierce_query: Query<&mut Pierce>,
    target_query: Query<&Faction, Without<Projectile>>,
    wall_query: Query<(), With<Wall>>,
    trigger_registry: Res<crate::abilities::TriggerRegistry>,
) {
    let mut despawned: HashSet<Entity> = HashSet::default();

    let Some(on_hit_trigger_id) = trigger_registry.get_id("on_hit") else {
        return;
    };

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

        let Ok((_projectile, source, proj_faction)) = projectile_query.get(projectile_entity) else {
            continue;
        };
        let Ok(target_faction) = target_query.get(other_entity) else {
            continue;
        };

        if proj_faction == target_faction {
            continue;
        }

        let on_hit_trigger = source.action.triggers.iter()
            .find(|t| t.trigger_type == on_hit_trigger_id);

        if let Some(trigger) = on_hit_trigger {
            let mut ctx = AbilityContext::new(
                source.caster,
                source.caster_faction,
                Vec3::ZERO,
            );
            ctx.set_param("target", ContextValue::Entity(other_entity));

            for action_def in &trigger.actions {
                action_events.write(ExecuteActionEvent {
                    action: action_def.clone(),
                    context: ctx.clone(),
                });
            }
        }

        let should_despawn = match pierce_query.get_mut(projectile_entity) {
            Err(_) => true,
            Ok(pierce) => match pierce.into_inner() {
                Pierce::Infinite => false,
                Pierce::Count(n) => {
                    *n = n.saturating_sub(1);
                    *n == 0
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

register_action!(SpawnProjectileHandler);
