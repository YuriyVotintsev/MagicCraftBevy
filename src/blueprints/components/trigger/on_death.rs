use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::context::TargetInfo;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::stats::{ComputedStats, Dead};

#[blueprint_component(SOURCE_ENTITY, SOURCE_POSITION)]
pub struct OnDeath {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_observer(on_death_observer);
}

fn on_death_observer(
    on: On<Add, Dead>,
    mut spawner: EntitySpawner,
    query: Query<(&OnDeath, &SpawnSource, &Transform)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    let entity = on.event_target();
    let Ok((on_death, source, transform)) = query.get(entity) else {
        return;
    };

    let source_pos = transform.translation.truncate();

    spawner.spawn_triggered(
        entity,
        source,
        TargetInfo::from_entity_and_position(entity, source_pos),
        TargetInfo::EMPTY,
        &on_death.entities,
        &stats_query,
        &transforms,
    );
}
