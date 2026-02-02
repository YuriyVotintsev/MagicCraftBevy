use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use magic_craft_macros::GenerateRaw;
use rand::Rng;

use crate::register_node;
use crate::abilities::NodeRegistry;
use crate::abilities::AbilityRegistry;
use crate::abilities::ParamValue;
use crate::abilities::ids::NodeTypeId;
use crate::abilities::context::Target;
use crate::abilities::AbilitySource;
use crate::abilities::events::ActionEventBase;
use crate::building_blocks::actions::ExecuteSpawnProjectileEvent;
use crate::building_blocks::triggers::on_collision::OnCollisionTrigger;
use crate::building_blocks::triggers::on_area::OnAreaTrigger;
use crate::building_blocks::triggers::TriggerParams;
use crate::building_blocks::NodeParams;
use crate::physics::{GameLayer, Wall};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::Faction;
use crate::{Growing, Lifetime};
use crate::GameState;

#[derive(Debug, Clone, Default, GenerateRaw)]
#[node(kind = Action)]
pub struct SpawnProjectileParams {
    #[raw(default = 800.0)]
    pub speed: ParamValue,
    #[raw(default = 15.0)]
    pub size: ParamValue,
    #[raw(default = 0.0)]
    pub spread: ParamValue,
    pub lifetime: Option<ParamValue>,
    pub start_size: Option<ParamValue>,
    pub end_size: Option<ParamValue>,
    pub pierce: Option<ParamValue>,
    pub color: Option<[f32; 4]>,
    #[raw(default = true)]
    pub collisions: bool,
}

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub enum Pierce {
    Count(u32),
    Infinite,
}

fn find_on_area_params<'a>(
    ability_registry: &'a AbilityRegistry,
    base: &ActionEventBase,
) -> Option<&'a crate::building_blocks::triggers::on_area::OnAreaParams> {
    let ability = ability_registry.get(base.ability_id)?;
    let action_node = ability.get_node(base.node_id)?;
    for &child_id in &action_node.children {
        let child = ability.get_node(child_id)?;
        if let NodeParams::Trigger(TriggerParams::OnAreaParams(params)) = &child.params {
            return Some(params);
        }
    }
    None
}

#[derive(Default)]
struct CachedTriggerIds {
    collision_id: Option<NodeTypeId>,
    area_id: Option<NodeTypeId>,
}

fn execute_spawn_projectile_action(
    mut commands: Commands,
    mut action_events: MessageReader<ExecuteSpawnProjectileEvent>,
    node_registry: Res<NodeRegistry>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
    mut cached_ids: Local<CachedTriggerIds>,
) {
    let collision_id = *cached_ids.collision_id.get_or_insert_with(|| {
        node_registry.get_id("OnCollisionParams")
            .expect("OnCollisionParams not registered")
    });
    let area_id = *cached_ids.area_id.get_or_insert_with(|| {
        node_registry.get_id("OnAreaParams")
            .expect("OnAreaParams not registered")
    });

    for event in action_events.read() {
        let caster_stats = stats_query
            .get(event.base.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let speed = event.params.speed.evaluate_f32(&caster_stats);
        let size = event.params.size.evaluate_f32(&caster_stats);
        let spread = event.params.spread.evaluate_f32(&caster_stats);
        let lifetime = event.params.lifetime.as_ref().map(|p| p.evaluate_f32(&caster_stats));
        let start_size = event.params.start_size.as_ref().map(|p| p.evaluate_f32(&caster_stats));
        let end_size = event.params.end_size.as_ref().map(|p| p.evaluate_f32(&caster_stats));
        let pierce = event.params.pierce.as_ref().map(|p| Pierce::Count(p.evaluate_i32(&caster_stats) as u32));

        let base_direction = match event.base.context.target {
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

        let sprite_color = event.params.color
            .map(|c| Color::srgba(c[0], c[1], c[2], c[3]))
            .unwrap_or(Color::srgb(1.0, 0.5, 0.0));

        let mut entity_commands = commands.spawn((
            Name::new("Projectile"),
            Projectile,
            AbilitySource::new(
                event.base.ability_id,
                event.base.node_id,
                event.base.context.caster,
                event.base.context.caster_faction,
            ),
            event.base.context.caster_faction,
            Collider::circle(initial_size / 2.0),
            Sensor,
            RigidBody::Kinematic,
            LinearVelocity(velocity),
            Sprite {
                color: sprite_color,
                custom_size: Some(Vec2::splat(initial_size)),
                ..default()
            },
            Transform::from_translation(event.base.context.source.as_point().unwrap_or(Vec3::ZERO)),
        ));

        if event.params.collisions {
            let projectile_layers = match event.base.context.caster_faction {
                Faction::Player => CollisionLayers::new(
                    GameLayer::PlayerProjectile,
                    [GameLayer::Enemy, GameLayer::Wall],
                ),
                Faction::Enemy => CollisionLayers::new(
                    GameLayer::EnemyProjectile,
                    [GameLayer::Player, GameLayer::Wall],
                ),
            };
            entity_commands.insert((CollisionEventsEnabled, projectile_layers));
        }

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

        if event.base.child_triggers.contains(&collision_id) {
            entity_commands.insert(OnCollisionTrigger);
        }

        if event.base.child_triggers.contains(&area_id) {
            if let Some(on_area_params) = find_on_area_params(&ability_registry, &event.base) {
                let radius = on_area_params.radius.evaluate_f32(&caster_stats);
                let interval = on_area_params.interval.as_ref().map(|i| i.evaluate_f32(&caster_stats));
                entity_commands.insert(OnAreaTrigger::with_interval(radius, interval));
            }
        }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        execute_spawn_projectile_action
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(
        Update,
        projectile_collision_physics.in_set(GameSet::AbilityExecution),
    );
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

register_node!(SpawnProjectileParams);
