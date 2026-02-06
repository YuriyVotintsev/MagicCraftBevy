use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::AbilitySource;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION)]
pub struct Interval {
    pub interval: ScalarExpr,
    #[raw(default = false)]
    pub skip_first: bool,
    pub entities: Vec<EntityDef>,
}

#[derive(Component, Default)]
pub struct IntervalTimer {
    pub timer: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_interval_timer, interval_system)
            .chain()
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_interval_timer(
    mut commands: Commands,
    query: Query<(Entity, &Interval), Added<Interval>>,
) {
    for (entity, interval) in &query {
        let initial_timer = if interval.skip_first { interval.interval } else { 0.0 };
        commands.entity(entity).insert(IntervalTimer { timer: initial_timer });
    }
}

fn interval_system(
    time: Res<Time>,
    mut commands: Commands,
    mut ability_query: Query<(&AbilitySource, &Interval, &mut IntervalTimer)>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
) {
    let delta = time.delta_secs();

    for (source, interval, mut timer) in &mut ability_query {
        timer.timer -= delta;
        if timer.timer > 0.0 { continue }

        let caster_entity = source.caster.entity.unwrap();
        let Ok(transform) = owner_query.get(caster_entity) else { continue };

        let caster_stats = stats_query
            .get(caster_entity)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = transform.translation.truncate();
        let source_info = TargetInfo::from_entity_and_position(caster_entity, caster_pos);

        let spawn_source = AbilitySource {
            ability_id: source.ability_id,
            caster: TargetInfo::from_entity_and_position(caster_entity, caster_pos),
            caster_faction: source.caster_faction,
            source: source_info,
            target: TargetInfo::EMPTY,
            index: 0,
            count: 1,
        };

        for entity_def in &interval.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_source, caster_stats, None, None, None);
        }

        timer.timer = interval.interval;
    }
}
