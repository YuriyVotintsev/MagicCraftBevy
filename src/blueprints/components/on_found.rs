use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::context::TargetInfo;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::ComputedStats;

use super::find_nearest_enemy::FoundTarget;

#[blueprint_component(SOURCE_POSITION, TARGET_ENTITY, TARGET_POSITION)]
pub struct OnFound {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        on_found_trigger_system
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn on_found_trigger_system(
    mut spawner: EntitySpawner,
    query: Query<(Entity, &OnFound, &FoundTarget, &SpawnSource)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    for (entity, trigger, found, source) in &query {
        let found_pos = found.1.truncate();
        spawner.spawn_triggered(
            entity,
            source,
            TargetInfo::from_position(found_pos),
            TargetInfo::from_entity_and_position(found.0, found_pos),
            &trigger.entities,
            &stats_query,
            &transforms,
        );

        spawner.commands.entity(entity).despawn();
    }
}
