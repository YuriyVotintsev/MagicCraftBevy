use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::{AbilityInputs, AbilitySource, TargetInfo};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::GameState;

#[ability_component(SOURCE_ENTITY, SOURCE_POSITION, TARGET_DIRECTION)]
pub struct WhileHeld {
    pub interval: ScalarExpr,
    pub entities: Vec<EntityDef>,
}

#[derive(Component, Default)]
pub struct WhileHeldTimer {
    pub timer: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_while_held_timer, while_held_system)
            .chain()
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_while_held_timer(
    mut commands: Commands,
    query: Query<Entity, Added<WhileHeld>>,
) {
    for entity in &query {
        commands.entity(entity).insert(WhileHeldTimer::default());
    }
}

fn while_held_system(
    time: Res<Time>,
    mut commands: Commands,
    mut ability_query: Query<(&AbilitySource, &WhileHeld, &mut WhileHeldTimer)>,
    owner_query: Query<(&AbilityInputs, &Transform)>,
    stats_query: Query<&ComputedStats>,
) {
    let delta = time.delta_secs();

    for (source, while_held, mut timer) in &mut ability_query {
        let caster_entity = source.caster.entity.unwrap();
        let Ok((inputs, transform)) = owner_query.get(caster_entity) else {
            continue;
        };

        let Some(input) = inputs.get(source.ability_id) else { continue };

        if !input.pressed {
            timer.timer = 0.0;
            continue;
        }

        timer.timer -= delta;
        if timer.timer > 0.0 { continue }

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

        for entity_def in &while_held.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_source, caster_stats, None, None, None);
        }

        timer.timer = while_held.interval;
    }
}
