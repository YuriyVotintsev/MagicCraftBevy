use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

use super::events::ActivateAbilityEvent;
use super::node::AbilityRegistry;
use super::spawn::{spawn_entity_def, SpawnContext};

pub fn ability_spawner(
    mut commands: Commands,
    mut activate_events: MessageReader<ActivateAbilityEvent>,
    ability_registry: Res<AbilityRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in activate_events.read() {
        let Some(ability_def) = ability_registry.get(event.ability_id) else {
            continue;
        };

        let caster_stats = stats_query
            .get(event.context.caster)
            .unwrap_or(&DEFAULT_STATS);

        let ctx = SpawnContext {
            ability_id: event.ability_id,
            caster: event.context.caster,
            caster_faction: event.context.caster_faction,
            source: event.context.source,
            target: event.context.target,
            stats: caster_stats,
            index: 0,
            count: 1,
        };

        for entity_def in &ability_def.entities {
            spawn_entity_def(&mut commands, entity_def, &ctx);
        }
    }
}

pub fn register_spawner_systems(app: &mut App) {
    app.add_systems(
        Update,
        ability_spawner
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}
