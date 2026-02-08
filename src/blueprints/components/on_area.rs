use bevy::prelude::*;
use avian2d::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::context::TargetInfo;
use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::stats::ComputedStats;

#[blueprint_component(SOURCE_ENTITY, SOURCE_POSITION, TARGET_ENTITY, TARGET_POSITION)]
pub struct OnArea {
    pub size: ScalarExpr,
    pub interval: Option<ScalarExpr>,
    pub entities: Vec<EntityDef>,
}

#[derive(Component, Default)]
pub struct OnAreaTimer {
    pub timer: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_on_area_timer, on_area_trigger_system)
            .chain()
            .in_set(GameSet::AbilityExecution),
    );
}

fn init_on_area_timer(
    mut commands: Commands,
    query: Query<Entity, Added<OnArea>>,
) {
    for entity in &query {
        commands.entity(entity).insert(OnAreaTimer::default());
    }
}

fn on_area_trigger_system(
    mut spawner: EntitySpawner,
    time: Res<Time>,
    mut query: Query<(Entity, &OnArea, &mut OnAreaTimer, &SpawnSource, &Transform)>,
    spatial_query: SpatialQuery,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    let dt = time.delta_secs();

    for (entity, trigger, mut timer, source, transform) in &mut query {
        timer.timer += dt;

        if let Some(interval) = trigger.interval {
            if timer.timer < interval {
                continue;
            }
            timer.timer = 0.0;
        }

        let position = transform.translation.truncate();

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(trigger.size / 2.0);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        let source_info = TargetInfo::from_entity_and_position(entity, position);

        for target_entity in hits {
            let target_pos = transforms.get(target_entity)
                .map(|t| t.translation.truncate())
                .unwrap_or(Vec2::ZERO);

            spawner.spawn_triggered(
                entity,
                source,
                source_info,
                TargetInfo::from_entity_and_position(target_entity, target_pos),
                &trigger.entities,
                &stats_query,
                &transforms,
            );
        }

        if trigger.interval.is_none() {
            spawner.commands.entity(entity).remove::<OnArea>();
        }
    }
}
