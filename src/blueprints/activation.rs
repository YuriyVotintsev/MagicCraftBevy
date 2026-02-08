use bevy::prelude::*;

use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::wave::{WaveEnemy, WavePhase};
use super::core_components::{AbilityCooldown, AbilityInput};
use super::context::TargetInfo;
use super::registry::BlueprintRegistry;
use super::spawn::EntitySpawner;
use super::SpawnSource;

pub fn ability_activation_system(
    time: Res<Time>,
    mut spawner: EntitySpawner,
    mut ability_query: Query<(&mut AbilityInput, &mut AbilityCooldown, &SpawnSource)>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
    wave_enemy_query: Query<(), With<WaveEnemy>>,
    blueprint_registry: Res<BlueprintRegistry>,
) {
    let delta = time.delta_secs();

    for (mut input, mut cd, source) in &mut ability_query {
        if cd.timer > 0.0 {
            cd.timer = (cd.timer - delta).max(0.0);
        }

        let should_fire = input.pressed && cd.timer <= 0.0;
        if input.pressed {
            input.pressed = false;
        }

        if !should_fire {
            continue;
        }

        let Some(caster_entity) = source.caster.entity else { continue };
        let Ok(transform) = owner_query.get(caster_entity) else {
            continue;
        };

        let Some(blueprint_def) = blueprint_registry.get(source.blueprint_id) else {
            continue;
        };

        let caster_stats = stats_query
            .get(caster_entity)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = transform.translation.truncate();
        let source_info = TargetInfo::from_entity_and_position(caster_entity, caster_pos);

        let spawn_source = SpawnSource {
            blueprint_id: source.blueprint_id,
            caster: TargetInfo::from_entity_and_position(caster_entity, caster_pos),
            caster_faction: source.caster_faction,
            source: source_info,
            target: input.target,
            index: 0,
            count: 1,
        };

        let is_wave_spawn = wave_enemy_query.contains(caster_entity);

        for entity_def in &blueprint_def.entities {
            let spawned = spawner.spawn(entity_def, &spawn_source, caster_stats);

            if is_wave_spawn {
                for &spawned_entity in &spawned {
                    spawner.commands.entity(spawned_entity).insert((
                        WaveEnemy,
                        DespawnOnExit(WavePhase::Combat),
                    ));
                }
            }
        }

        cd.timer = blueprint_def.cooldown.eval(&spawn_source, caster_stats);
    }
}
