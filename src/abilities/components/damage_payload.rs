use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::eval_context::EvalContext;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, PendingDamage, DEFAULT_STATS};
use crate::GameState;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub amount: ScalarExprRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub amount: ScalarExpr,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            amount: self.amount.resolve(stat_registry),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    (raw.amount.required_fields(), None)
}

#[derive(Component)]
pub struct DamagePayload {
    pub amount: ScalarExpr,
    pub target: Entity,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    if let Some(target) = ctx.target.entity {
        commands.insert(DamagePayload {
            amount: def.amount.clone(),
            target,
        });
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        process_damage_payloads
            .in_set(GameSet::Damage)
            .run_if(in_state(GameState::Playing)),
    );
}

fn process_damage_payloads(
    mut commands: Commands,
    query: Query<(Entity, &DamagePayload, &crate::abilities::AbilitySource)>,
    stats_query: Query<&ComputedStats>,
) {
    for (entity, payload, source) in &query {
        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);
        let amount = payload.amount.eval(&EvalContext::stats_only(caster_stats));

        if let Ok(mut target_commands) = commands.get_entity(payload.target) {
            target_commands.insert(PendingDamage(amount));
        }

        commands.entity(entity).despawn();
    }
}
