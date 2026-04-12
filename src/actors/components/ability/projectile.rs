use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use avian3d::prelude::*;

use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::Faction;

#[derive(Component)]
pub struct Projectile;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_projectile);
    app.add_systems(
        Update,
        projectile_collision_physics.in_set(GameSet::AbilityExecution),
    );
}

fn init_projectile(mut commands: Commands, query: Query<Entity, Added<Projectile>>) {
    for entity in &query {
        commands.entity(entity).insert(Name::new("Projectile"));
    }
}

pub fn projectile_collision_physics(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    projectile_query: Query<(&Projectile, &Faction), Without<Wall>>,
    target_query: Query<&Faction, Without<Projectile>>,
    wall_query: Query<(), With<Wall>>,
    mut despawned: Local<HashSet<Entity>>,
) {
    despawned.clear();
    for event in collision_events.read() {
        let (projectile_entity, other_entity) =
            if projectile_query.contains(event.collider1) { (event.collider1, event.collider2) }
            else if projectile_query.contains(event.collider2) { (event.collider2, event.collider1) }
            else { continue };

        if despawned.contains(&projectile_entity) { continue }

        if wall_query.contains(other_entity) {
            if let Ok(mut ec) = commands.get_entity(projectile_entity) {
                ec.despawn();
                despawned.insert(projectile_entity);
            }
            continue;
        }

        if projectile_query.contains(other_entity) { continue }

        let Ok((_, proj_faction)) = projectile_query.get(projectile_entity) else { continue };
        let Ok(target_faction) = target_query.get(other_entity) else { continue };
        if proj_faction == target_faction { continue }

        if let Ok(mut ec) = commands.get_entity(projectile_entity) {
            ec.despawn();
            despawned.insert(projectile_entity);
        }
    }
}
