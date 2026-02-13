use bevy::prelude::*;

use crate::stats::{ComputedStats, DEFAULT_STATS};
use crate::wave::{WaveEnemy, WavePhase};
use super::core_components::{BlueprintActivationCooldown, BlueprintActivationInput, TrackedSpawns};
use super::context::TargetInfo;
use super::registry::BlueprintRegistry;
use super::spawn::EntitySpawner;
use super::SpawnSource;

pub fn blueprint_activation_system(
    time: Res<Time>,
    mut spawner: EntitySpawner,
    mut blueprint_query: Query<(Entity, &mut BlueprintActivationInput, &mut BlueprintActivationCooldown, &SpawnSource)>,
    tracked_query: Query<&TrackedSpawns>,
    owner_query: Query<&Transform>,
    stats_query: Query<&ComputedStats>,
    wave_enemy_query: Query<(), With<WaveEnemy>>,
    blueprint_registry: Res<BlueprintRegistry>,
) {
    let delta = time.delta_secs();

    for (bp_entity, mut input, mut cd, source) in &mut blueprint_query {
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

        let track = blueprint_def.entities.iter()
            .any(|e| e.count.as_ref().is_some_and(|c| c.uses_recalc()));

        if track {
            if let Ok(tracked) = tracked_query.get(bp_entity) {
                for &e in &tracked.entities {
                    spawner.commands.entity(e).despawn();
                }
            }
        }

        let is_wave_spawn = wave_enemy_query.contains(caster_entity);

        let mut all_spawned = Vec::new();
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

            if track {
                all_spawned.extend(spawned);
            }
        }

        if track {
            spawner.commands.entity(bp_entity).insert(TrackedSpawns { entities: all_spawned });
        }

        cd.timer = blueprint_def.cooldown.eval(&spawn_source, caster_stats);
    }
}

pub fn respawn_on_count_change(
    mut blueprint_query: Query<(&SpawnSource, &mut BlueprintActivationCooldown, &TrackedSpawns)>,
    stats_query: Query<&ComputedStats, Changed<ComputedStats>>,
    blueprint_registry: Res<BlueprintRegistry>,
) {
    for (source, mut cd, tracked) in &mut blueprint_query {
        let Some(caster_entity) = source.caster.entity else { continue };
        let Ok(stats) = stats_query.get(caster_entity) else { continue };
        let Some(blueprint_def) = blueprint_registry.get(source.blueprint_id) else { continue };

        let new_count: usize = blueprint_def.entities.iter()
            .map(|e| e.count.as_ref()
                .map(|c| c.eval(source, stats).max(1.0) as usize)
                .unwrap_or(1))
            .sum();

        if new_count != tracked.entities.len() {
            cd.timer = 0.0;
        }
    }
}
