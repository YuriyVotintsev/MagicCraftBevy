use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::{ComputedStats, DEFAULT_STATS};

use super::find_nearest_enemy::FoundTarget;

#[ability_component(SOURCE_POSITION, TARGET_ENTITY, TARGET_POSITION)]
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
    mut commands: Commands,
    query: Query<(Entity, &OnFound, &FoundTarget, &AbilitySource)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    for (entity, trigger, found, source) in &query {
        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = transforms.get(source.caster)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let found_pos = found.1.truncate();
        let source_info = TargetInfo::from_position(found_pos);
        let target_info = TargetInfo::from_entity_and_position(found.0, found_pos);

        let spawn_ctx = SpawnContext {
            ability_id: source.ability_id,
            caster: source.caster,
            caster_position: caster_pos,
            caster_faction: source.caster_faction,
            source: source_info,
            target: target_info,
            stats: caster_stats,
            index: 0,
            count: 1,
        };

        for entity_def in &trigger.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
        }

        commands.entity(entity).despawn();
    }
}
