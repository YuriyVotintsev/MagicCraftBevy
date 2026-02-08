use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use avian2d::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::spawn::EntitySpawner;
use crate::abilities::AbilitySource;
use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::Faction;
use crate::stats::ComputedStats;

use super::pierce::Pierce;
use super::projectile::Projectile;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION, TARGET_ENTITY, TARGET_POSITION)]
pub struct OnCollision {
    pub entities: Vec<EntityDef>,
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
    mut spawner: EntitySpawner,
    mut collision_events: MessageReader<CollisionStart>,
    hittable_query: Query<(&AbilitySource, &Faction, &Transform, &OnCollision)>,
    target_query: Query<(&Faction, &Transform), Without<OnCollision>>,
    wall_query: Query<(), With<Wall>>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
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

        let Ok((source, hittable_faction, hittable_transform, trigger)) = hittable_query.get(hittable_entity) else {
            continue;
        };
        let Ok((target_faction, target_transform)) = target_query.get(other_entity) else {
            continue;
        };

        if hittable_faction == target_faction {
            continue;
        }

        let source_pos = hittable_transform.translation.truncate();
        let target_pos = target_transform.translation.truncate();

        spawner.spawn_triggered(
            hittable_entity,
            source,
            TargetInfo::from_entity_and_position(hittable_entity, source_pos),
            TargetInfo::from_entity_and_position(other_entity, target_pos),
            &trigger.entities,
            &stats_query,
            &transforms,
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
                .is_ok_and(|p| p.count.is_none());

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
            Ok(mut pierce) => match pierce.count {
                None => false,
                Some(n) if n <= 1.0 => true,
                Some(n) => {
                    pierce.count = Some(n - 1.0);
                    false
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
