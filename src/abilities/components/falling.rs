use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::{ProvidedFields, TargetInfo};
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw, VecExpr, VecExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::{ComputedStats, DEFAULT_STATS};

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub height: ScalarExprRaw,
    pub duration: ScalarExprRaw,
    #[serde(default)]
    pub target: Option<VecExprRaw>,
    #[serde(default)]
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub height: ScalarExpr,
    pub duration: ScalarExpr,
    pub target: Option<VecExpr>,
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            height: self.height.resolve(stat_registry),
            duration: self.duration.resolve(stat_registry),
            target: self.target.as_ref().map(|t| t.resolve(stat_registry)),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let mut fields = raw.height.required_fields().union(raw.duration.required_fields());
    if let Some(ref target) = raw.target {
        fields = fields.union(target.required_fields());
    }
    let provided = ProvidedFields::SOURCE_POSITION;
    let nested = if raw.entities.is_empty() {
        None
    } else {
        Some((provided, raw.entities.as_slice()))
    };
    (fields, nested)
}

#[derive(Component)]
pub struct FallingProjectile {
    pub target_position: Vec2,
    pub height: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub caster_position: Vec2,
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let height = def.height.eval(&eval_ctx);
    let duration = def.duration.eval(&eval_ctx);

    let target_position = match &def.target {
        Some(target_expr) => target_expr.eval(&eval_ctx),
        None => ctx.target.position
            .or(ctx.source.position)
            .unwrap_or(Vec2::ZERO),
    };

    commands.insert(FallingProjectile {
        target_position,
        height,
        duration,
        elapsed: 0.0,
        caster_position: ctx.caster_position,
        entities: def.entities.clone(),
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        update_falling_projectiles
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn update_falling_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FallingProjectile, &AbilitySource, &mut Transform)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform, Without<FallingProjectile>>,
) {
    let dt = time.delta_secs();

    for (entity, mut falling, source, mut transform) in &mut query {
        falling.elapsed += dt;
        let t = (falling.elapsed / falling.duration).clamp(0.0, 1.0);
        let eased_t = t * t;

        let start_y = falling.target_position.y + falling.height;
        let current_y = start_y - (falling.height * eased_t);
        transform.translation.x = falling.target_position.x;
        transform.translation.y = current_y;

        if t >= 1.0 {
            let caster_stats = stats_query
                .get(source.caster)
                .unwrap_or(&DEFAULT_STATS);

            let caster_pos = transforms.get(source.caster)
                .map(|t| t.translation.truncate())
                .unwrap_or(falling.caster_position);

            let spawn_ctx = SpawnContext {
                ability_id: source.ability_id,
                caster: source.caster,
                caster_position: caster_pos,
                caster_faction: source.caster_faction,
                source: TargetInfo::from_position(falling.target_position),
                target: TargetInfo::EMPTY,
                stats: caster_stats,
                index: 0,
                count: 1,
            };

            for entity_def in &falling.entities {
                crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
            }

            commands.entity(entity).despawn();
        }
    }
}
