use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::{EntityDef, EntityDefRaw};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::{AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    #[serde(default)]
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

pub fn provided_fields() -> ProvidedFields {
    ProvidedFields::SOURCE_ENTITY
        .union(ProvidedFields::SOURCE_POSITION)
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let nested = if raw.entities.is_empty() {
        None
    } else {
        Some((provided_fields(), raw.entities.as_slice()))
    };
    (ProvidedFields::NONE, nested)
}

#[derive(Component)]
pub struct Once {
    pub triggered: bool,
    pub entities: Vec<EntityDef>,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, _ctx: &SpawnContext) {
    commands.insert(Once {
        triggered: false,
        entities: def.entities.clone(),
    });
}

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
    mut ability_query: Query<(Entity, &AbilitySource, &mut Once)>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
) {
    for (_entity, source, mut once) in &mut ability_query {
        if once.triggered { continue }

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

        once.triggered = true;
    }
}
