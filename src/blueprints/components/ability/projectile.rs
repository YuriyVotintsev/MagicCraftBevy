use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use avian2d::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::Faction;

use super::pierce::Pierce;

#[blueprint_component]
pub struct Projectile;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_projectile);
    app.add_systems(
        Update,
        projectile_collision_physics.in_set(GameSet::BlueprintExecution),
    );
}

fn init_projectile(mut commands: Commands, query: Query<Entity, Added<Projectile>>) {
    for entity in &query {
        commands.entity(entity).insert(Name::new("Projectile"));
    }
}

fn projectile_collision_physics(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    projectile_query: Query<(&Projectile, &Faction), Without<Wall>>,
    mut pierce_query: Query<&mut Pierce>,
    target_query: Query<&Faction, Without<Projectile>>,
    wall_query: Query<(), With<Wall>>,
    mut despawned: Local<HashSet<Entity>>,
) {
    despawned.clear();

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
