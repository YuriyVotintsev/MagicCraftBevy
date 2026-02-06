use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::AbilitySource;
use crate::stats::{ComputedStats, Dead, DEFAULT_STATS, StatCalculators, StatRegistry};
use crate::wave::WaveState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION)]
pub struct OnDeath {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_observer(on_death_observer);
}

fn on_death_observer(
    on: On<Add, Dead>,
    mut commands: Commands,
    query: Query<(&OnDeath, &AbilitySource, &Transform)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
    stat_registry: Option<Res<StatRegistry>>,
    calculators: Option<Res<StatCalculators>>,
    ability_registry: Res<crate::abilities::AbilityRegistry>,
    mut wave_state: ResMut<WaveState>,
) {
    let entity = on.event_target();
    let Ok((on_death, source, transform)) = query.get(entity) else {
        return;
    };

    let caster_entity = source.caster.entity.unwrap_or(entity);
    let caster_stats = stats_query
        .get(caster_entity)
        .unwrap_or(&DEFAULT_STATS);

    let caster_pos = transforms
        .get(caster_entity)
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let source_pos = transform.translation.truncate();
    let source_info = TargetInfo::from_entity_and_position(entity, source_pos);

    let spawn_source = AbilitySource {
        ability_id: source.ability_id,
        caster: TargetInfo::from_entity_and_position(caster_entity, caster_pos),
        caster_faction: source.caster_faction,
        source: source_info,
        target: TargetInfo::EMPTY,
        index: 0,
        count: 1,
    };

    for entity_def in &on_death.entities {
        let count = entity_def
            .count
            .as_ref()
            .map(|c| c.eval(&spawn_source, caster_stats).max(1.0) as u32)
            .unwrap_or(1);

        wave_state.extra_spawned += count;

        crate::abilities::spawn::spawn_entity_def(
            &mut commands,
            entity_def,
            &spawn_source,
            caster_stats,
            stat_registry.as_deref(),
            calculators.as_deref(),
            Some(&ability_registry),
        );
    }
}
