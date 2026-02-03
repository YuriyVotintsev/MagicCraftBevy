use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use avian2d::prelude::*;
use serde::Deserialize;

use crate::abilities::spawn::SpawnContext;
use crate::abilities::Target;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::Faction;
use crate::stats::{ComputedStats, DEFAULT_STATS, StatRegistry};

use super::pierce::Pierce;
use super::projectile::Projectile;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub entities: Vec<crate::abilities::entity_def::EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> Def {
        Def {
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

#[derive(Component)]
pub struct OnCollisionTrigger {
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, _ctx: &SpawnContext) {
    commands.insert(OnCollisionTrigger {
        entities: def.entities.clone(),
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (on_collision_trigger_system, projectile_collision_physics)
            .chain()
            .in_set(GameSet::AbilityExecution),
    );
}

fn on_collision_trigger_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    hittable_query: Query<(&AbilitySource, &Faction, &Transform, &OnCollisionTrigger)>,
    target_query: Query<&Faction, Without<OnCollisionTrigger>>,
    wall_query: Query<(), With<Wall>>,
    stats_query: Query<&ComputedStats>,
) {
    let mut processed: HashSet<(Entity, Entity)> = HashSet::default();

    for event in collision_events.read() {
        let (hittable_entity, other_entity) = if hittable_query.contains(event.collider1) {
            (event.collider1, event.collider2)
        } else if hittable_query.contains(event.collider2) {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if processed.contains(&(hittable_entity, other_entity)) {
            continue;
        }
        processed.insert((hittable_entity, other_entity));

        if wall_query.contains(other_entity) {
            continue;
        }

        if hittable_query.contains(other_entity) {
            continue;
        }

        let Ok((source, hittable_faction, transform, trigger)) = hittable_query.get(hittable_entity) else {
            continue;
        };
        let Ok(target_faction) = target_query.get(other_entity) else {
            continue;
        };

        if hittable_faction == target_faction {
            continue;
        }

        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let spawn_ctx = SpawnContext {
            ability_id: source.ability_id,
            caster: source.caster,
            caster_faction: source.caster_faction,
            source: Target::Point(transform.translation),
            target: Some(Target::Entity(other_entity)),
            stats: caster_stats,
            index: 0,
            count: 1,
        };

        for entity_def in &trigger.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
        }
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
