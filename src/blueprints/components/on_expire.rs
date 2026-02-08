use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::context::TargetInfo;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::schedule::GameSet;
use super::lifetime::Lifetime;
use crate::stats::ComputedStats;

#[blueprint_component(SOURCE_POSITION)]
pub struct OnExpire {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, on_expire_trigger_system.in_set(GameSet::AbilityExecution));
}

fn on_expire_trigger_system(
    mut spawner: EntitySpawner,
    query: Query<(Entity, &OnExpire, &SpawnSource, &Transform, &Lifetime)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    for (entity, trigger, source, transform, lifetime) in &query {
        if lifetime.remaining > 0.0 {
            continue;
        }

        let source_pos = transform.translation.truncate();
        spawner.spawn_triggered(
            entity,
            source,
            TargetInfo::from_position(source_pos),
            TargetInfo::EMPTY,
            &trigger.entities,
            &stats_query,
            &transforms,
        );

        spawner.commands.entity(entity).remove::<OnExpire>();
    }
}
