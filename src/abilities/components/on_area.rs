use bevy::prelude::*;
use avian2d::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::Target;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::stats::{ComputedStats, DEFAULT_STATS, StatRegistry};

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub radius: ParamValueRaw,
    #[serde(default)]
    pub interval: Option<ParamValueRaw>,
    pub entities: Vec<crate::abilities::entity_def::EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub radius: ParamValue,
    pub interval: Option<ParamValue>,
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> Def {
        Def {
            radius: resolve_param_value(&self.radius, stat_registry),
            interval: self.interval.as_ref().map(|i| resolve_param_value(i, stat_registry)),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

#[derive(Component)]
pub struct OnAreaTrigger {
    pub radius: f32,
    pub interval: Option<f32>,
    pub timer: f32,
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let radius = def.radius.evaluate_f32(ctx.stats);
    let interval = def.interval.as_ref().map(|i| i.evaluate_f32(ctx.stats));
    commands.insert(OnAreaTrigger {
        radius,
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
    mut query: Query<(Entity, &mut OnAreaTrigger, &AbilitySource, &Transform)>,
    spatial_query: SpatialQuery,
    stats_query: Query<&ComputedStats>,
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
        let shape = Collider::circle(trigger.radius);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        for target_entity in hits {
            let spawn_ctx = SpawnContext {
                ability_id: source.ability_id,
                caster: source.caster,
                caster_faction: source.caster_faction,
                source: Target::Point(transform.translation),
                target: Some(Target::Entity(target_entity)),
                stats: caster_stats,
                index: 0,
                count: 1,
            };

            for entity_def in &trigger.entities {
                crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
            }
        }

        if trigger.interval.is_none() {
            commands.entity(entity).remove::<OnAreaTrigger>();
        }
    }
}
