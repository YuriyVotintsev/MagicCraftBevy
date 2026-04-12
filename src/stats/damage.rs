use bevy::prelude::*;

use crate::actors::components::common::health::Health;
use crate::hit_flash::HitFlash;
use crate::wave::InvulnerableStack;

use super::{ComputedStats, PendingDamage, StatRegistry};

pub fn apply_pending_damage(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    mut pending: MessageReader<PendingDamage>,
    mut query: Query<(&mut Health, &ComputedStats)>,
    stats_query: Query<&ComputedStats>,
    invulnerable: Query<(), With<InvulnerableStack>>,
) {
    let max_life_id = stat_registry.get("max_life");
    let crit_chance_id = stat_registry.get("crit_chance");
    let crit_multiplier_id = stat_registry.get("crit_multiplier");

    for hit in pending.read() {
        if invulnerable.get(hit.target).is_ok() {
            continue;
        }

        let Ok((mut health, stats)) = query.get_mut(hit.target) else {
            continue;
        };

        let mut amount = hit.amount;

        if let Some(source_entity) = hit.source {
            if let Ok(source_stats) = stats_query.get(source_entity) {
                let chance = crit_chance_id.map(|id| source_stats.get(id)).unwrap_or(0.0);
                if rand::random::<f32>() < chance {
                    let multiplier = crit_multiplier_id.map(|id| source_stats.get(id)).unwrap_or(1.5);
                    amount *= multiplier;
                }
            }
        }

        let max = max_life_id.map(|id| stats.get(id)).unwrap_or(1.0).max(1.0);
        health.current = (health.current - amount).clamp(0.0, max);

        if let Ok(mut entity_commands) = commands.get_entity(hit.target) {
            entity_commands.insert(HitFlash::new());
        }
    }
}
