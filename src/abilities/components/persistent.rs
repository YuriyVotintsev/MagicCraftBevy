use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::spawn::SpawnContext;
use crate::abilities::{AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION)]
pub struct Persistent {
    pub entities: Vec<EntityDef>,
}

#[derive(Component)]
pub struct PersistentState {
    pub spawned: Vec<Entity>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_persistent, update_persistent)
            .chain()
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_persistent(
    mut commands: Commands,
    ability_query: Query<(Entity, &AbilitySource, &Persistent), Without<PersistentState>>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
) {
    for (entity, source, persistent) in &ability_query {
        let Ok(transform) = owner_query.get(source.caster) else {
            continue;
        };

        let caster_stats = stats_query.get(source.caster).unwrap_or(&DEFAULT_STATS);

        let spawned = spawn_persistent_entities(
            &mut commands, source, persistent, transform, caster_stats,
        );

        commands.entity(entity).insert(PersistentState { spawned });
    }
}

fn update_persistent(
    mut commands: Commands,
    mut ability_query: Query<(&AbilitySource, &Persistent, &mut PersistentState)>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats, Changed<ComputedStats>>,
    all_stats_query: Query<&ComputedStats>,
) {
    for (source, persistent, mut state) in &mut ability_query {
        if stats_query.get(source.caster).is_err() {
            continue;
        }

        let Ok(transform) = owner_query.get(source.caster) else {
            continue;
        };

        for &spawned_entity in &state.spawned {
            if let Ok(mut ec) = commands.get_entity(spawned_entity) {
                ec.despawn();
            }
        }

        let caster_stats = all_stats_query.get(source.caster).unwrap_or(&DEFAULT_STATS);

        state.spawned = spawn_persistent_entities(
            &mut commands, source, persistent, transform, caster_stats,
        );
    }
}

fn spawn_persistent_entities(
    commands: &mut Commands,
    source: &AbilitySource,
    persistent: &Persistent,
    transform: &Transform,
    caster_stats: &ComputedStats,
) -> Vec<Entity> {
    let source_info = TargetInfo::from_entity_and_position(
        source.caster,
        transform.translation.truncate(),
    );

    let spawn_ctx = SpawnContext {
        ability_id: source.ability_id,
        caster: source.caster,
        caster_position: transform.translation.truncate(),
        caster_faction: source.caster_faction,
        source: source_info,
        target: TargetInfo::EMPTY,
        stats: caster_stats,
        index: 0,
        count: 1,
    };

    let mut spawned = Vec::new();
    for entity_def in &persistent.entities {
        spawned.extend(crate::abilities::spawn::spawn_entity_def(commands, entity_def, &spawn_ctx, None, None, None));
    }
    spawned
}
