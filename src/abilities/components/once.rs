use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::spawn::SpawnContext;
use crate::abilities::{AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION)]
pub struct Once {
    pub entities: Vec<EntityDef>,
}

#[derive(Component)]
pub struct OnceTriggered;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        once_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

fn once_system(
    mut commands: Commands,
    ability_query: Query<(Entity, &AbilitySource, &Once), Without<OnceTriggered>>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
) {
    for (entity, source, once) in &ability_query {
        let Ok(transform) = owner_query.get(source.caster) else {
            continue;
        };

        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let source_info = TargetInfo::from_entity_and_position(source.caster, transform.translation.truncate());
        let target_info = TargetInfo::EMPTY;

        let spawn_ctx = SpawnContext {
            ability_id: source.ability_id,
            caster: source.caster,
            caster_position: transform.translation.truncate(),
            caster_faction: source.caster_faction,
            source: source_info,
            target: target_info,
            stats: caster_stats,
            index: 0,
            count: 1,
        };

        for entity_def in &once.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
        }

        commands.entity(entity).insert(OnceTriggered);
    }
}
