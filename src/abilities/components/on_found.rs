use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::spawn::SpawnContext;
use crate::abilities::Target;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::{ComputedStats, DEFAULT_STATS, StatRegistry};

use super::find_nearest_enemy::FoundTarget;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub entities: Vec<crate::abilities::entity_def::EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> Def {
        Def {
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

#[derive(Component)]
pub struct OnFoundTrigger {
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, _ctx: &SpawnContext) {
    commands.insert(OnFoundTrigger {
        entities: def.entities.clone(),
    });
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
    query: Query<(Entity, &OnFoundTrigger, &FoundTarget, &AbilitySource, &Transform)>,
    stats_query: Query<&ComputedStats>,
) {
    for (entity, trigger, found, source, transform) in &query {
        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let spawn_ctx = SpawnContext {
            ability_id: source.ability_id,
            caster: source.caster,
            caster_faction: source.caster_faction,
            source: Target::Point(transform.translation),
            target: Some(Target::Point(found.1)),
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
