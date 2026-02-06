use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::{AbilityRegistry, AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, StatCalculators, StatRegistry, DEFAULT_STATS};
use crate::GameState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION)]
pub struct Once {
    pub entities: Vec<EntityDef>,
}

#[derive(Component)]
pub struct OnceTriggered;

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
    ability_query: Query<(Entity, &AbilitySource, &Once), Without<OnceTriggered>>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
    stat_registry: Option<Res<StatRegistry>>,
    calculators: Option<Res<StatCalculators>>,
    ability_registry: Res<AbilityRegistry>,
) {
    for (entity, source, once) in &ability_query {
        let caster_entity = source.caster.entity.unwrap();
        let Ok(transform) = owner_query.get(caster_entity) else {
            continue;
        };

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

        for entity_def in &once.entities {
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

        commands.entity(entity).insert(OnceTriggered);
    }
}
