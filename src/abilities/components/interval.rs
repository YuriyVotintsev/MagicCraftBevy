use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::{EntityDef, EntityDefRaw};
use crate::abilities::eval_context::EvalContext;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::{AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub interval: ScalarExprRaw,
    #[serde(default)]
    pub skip_first: bool,
    #[serde(default)]
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub interval: ScalarExpr,
    pub skip_first: bool,
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            interval: self.interval.resolve(stat_registry),
            skip_first: self.skip_first,
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
    (raw.interval.required_fields(), nested)
}

#[derive(Component)]
pub struct Interval {
    pub interval: ScalarExpr,
    pub timer: f32,
    pub skip_first: bool,
    pub activated: bool,
    pub entities: Vec<EntityDef>,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, _ctx: &SpawnContext) {
    commands.insert(Interval {
        interval: def.interval.clone(),
        timer: 0.0,
        skip_first: def.skip_first,
        activated: false,
        entities: def.entities.clone(),
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        interval_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

fn interval_system(
    time: Res<Time>,
    mut commands: Commands,
    mut ability_query: Query<(Entity, &AbilitySource, &mut Interval)>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
) {
    let delta = time.delta_secs();

    for (_entity, source, mut interval) in &mut ability_query {
        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        if interval.skip_first && !interval.activated {
            interval.timer = interval.interval.eval(&EvalContext::stats_only(caster_stats));
            interval.activated = true;
            continue;
        }

        interval.timer -= delta;
        if interval.timer > 0.0 { continue }

        let Ok(transform) = owner_query.get(source.caster) else { continue };

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

        for entity_def in &interval.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
        }

        interval.timer = interval.interval.eval(&EvalContext::stats_only(caster_stats));
    }
}
