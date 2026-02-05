use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::{ProvidedFields, TargetInfo};
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::schedule::GameSet;
use super::lifetime::Lifetime;
use crate::stats::{ComputedStats, DEFAULT_STATS};

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
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

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let provided = ProvidedFields::SOURCE_POSITION;
    let nested = if raw.entities.is_empty() {
        None
    } else {
        Some((provided, raw.entities.as_slice()))
    };
    (ProvidedFields::NONE, nested)
}

#[derive(Component)]
pub struct OnExpireTrigger {
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, _ctx: &SpawnContext) {
    commands.insert(OnExpireTrigger {
        entities: def.entities.clone(),
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, on_expire_trigger_system.in_set(GameSet::AbilityExecution));
}

fn on_expire_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &OnExpireTrigger, &AbilitySource, &Transform, &Lifetime)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    for (entity, trigger, source, transform, lifetime) in &query {
        if lifetime.remaining > 0.0 {
            continue;
        }

        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = transforms.get(source.caster)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let source_pos = transform.translation.truncate();
        let source_info = TargetInfo::from_position(source_pos);

        let spawn_ctx = SpawnContext {
            ability_id: source.ability_id,
            caster: source.caster,
            caster_position: caster_pos,
            caster_faction: source.caster_faction,
            source: source_info,
            target: TargetInfo::EMPTY,
            stats: caster_stats,
            index: 0,
            count: 1,
        };

        for entity_def in &trigger.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
        }

        commands.entity(entity).remove::<OnExpireTrigger>();
    }
}
