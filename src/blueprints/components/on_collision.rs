use bevy::prelude::*;
use avian2d::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::context::TargetInfo;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::Faction;
use crate::stats::ComputedStats;

#[blueprint_component(SOURCE_ENTITY, SOURCE_POSITION, TARGET_ENTITY, TARGET_POSITION)]
pub struct OnCollision {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        on_collision_trigger_system.in_set(GameSet::AbilityExecution),
    );
}

fn on_collision_trigger_system(
    mut spawner: EntitySpawner,
    mut collision_events: MessageReader<CollisionStart>,
    hittable_query: Query<(&SpawnSource, &Faction, &Transform, &OnCollision)>,
    target_query: Query<(&Faction, &Transform), Without<OnCollision>>,
    wall_query: Query<(), With<Wall>>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
    mut processed: Local<bevy::platform::collections::HashSet<(Entity, Entity)>>,
) {
    processed.clear();

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
