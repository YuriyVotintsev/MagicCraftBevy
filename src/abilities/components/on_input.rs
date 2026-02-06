use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::{AbilityInputs, AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION, TARGET_DIRECTION)]
pub struct OnInput {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        on_input_system
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

fn on_input_system(
    mut commands: Commands,
    ability_query: Query<(Entity, &AbilitySource, &OnInput)>,
    owner_query: Query<(&AbilityInputs, &Transform)>,
    stats_query: Query<&ComputedStats>,
) {
    for (_entity, source, on_input) in &ability_query {
        let caster_entity = source.caster.entity.unwrap();
        let Ok((inputs, transform)) = owner_query.get(caster_entity) else {
            continue;
        };

        let Some(input) = inputs.get(source.ability_id) else { continue };
        if !input.just_pressed { continue }

        let caster_stats = stats_query
            .get(caster_entity)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = transform.translation.truncate();
        let source_info = TargetInfo::from_entity_and_position(caster_entity, caster_pos);
        let target_info = TargetInfo::from_direction(input.direction.truncate());

        let spawn_source = AbilitySource {
            ability_id: source.ability_id,
            caster: TargetInfo::from_entity_and_position(caster_entity, caster_pos),
            caster_faction: source.caster_faction,
            source: source_info,
            target: target_info,
            index: 0,
            count: 1,
        };

        for entity_def in &on_input.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_source, caster_stats, None, None, None);
        }
    }
}
