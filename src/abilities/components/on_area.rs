use bevy::prelude::*;
use avian2d::prelude::*;
use serde::Deserialize;

use crate::abilities::context::{ProvidedFields, TargetInfo};
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::stats::{ComputedStats, DEFAULT_STATS};

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub size: ScalarExprRaw,
    #[serde(default)]
    pub interval: Option<ScalarExprRaw>,
    pub entities: Vec<EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub size: ScalarExpr,
    pub interval: Option<ScalarExpr>,
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            size: self.size.resolve(stat_registry),
            interval: self.interval.as_ref().map(|i| i.resolve(stat_registry)),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let mut fields = raw.size.required_fields();
    if let Some(ref interval) = raw.interval {
        fields = fields.union(interval.required_fields());
    }
    let provided = ProvidedFields::SOURCE_ENTITY
        .union(ProvidedFields::SOURCE_POSITION)
        .union(ProvidedFields::TARGET_ENTITY)
        .union(ProvidedFields::TARGET_POSITION);
    let nested = if raw.entities.is_empty() {
        None
    } else {
        Some((provided, raw.entities.as_slice()))
    };
    (fields, nested)
}

#[derive(Component)]
pub struct OnArea {
    pub size: f32,
    pub interval: Option<f32>,
    pub timer: f32,
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let size = def.size.eval(&eval_ctx);
    let interval = def.interval.as_ref().map(|i| i.eval(&eval_ctx));
    commands.insert(OnArea {
        size,
        interval,
        timer: 0.0,
        entities: def.entities.clone(),
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, on_area_trigger_system.in_set(GameSet::AbilityExecution));
}

fn on_area_trigger_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut OnArea, &AbilitySource, &Transform)>,
    spatial_query: SpatialQuery,
    stats_query: Query<&ComputedStats>,
    target_transforms: Query<&Transform>,
) {
    let dt = time.delta_secs();

    for (entity, mut trigger, source, transform) in &mut query {
        trigger.timer += dt;

        if let Some(interval) = trigger.interval {
            if trigger.timer < interval {
                continue;
            }
            trigger.timer = 0.0;
        }

        let position = transform.translation.truncate();

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(trigger.size / 2.0);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = target_transforms.get(source.caster)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let source_info = TargetInfo::from_entity_and_position(entity, position);

        for target_entity in hits {
            let target_pos = target_transforms.get(target_entity)
                .map(|t| t.translation.truncate())
                .unwrap_or(Vec2::ZERO);
            let target_info = TargetInfo::from_entity_and_position(target_entity, target_pos);

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
        }

        if trigger.interval.is_none() {
            commands.entity(entity).remove::<OnArea>();
        }
    }
}
