use bevy::prelude::*;

use crate::blueprints::components::common::health::Health;
use crate::hit_flash::HitFlash;
use crate::wave::InvulnerableStack;
use crate::Faction;

use super::{ComputedStats, PendingDamage, StatRegistry};

#[derive(Message)]
pub struct DamageEvent {
    pub position: Vec3,
    pub amount: f32,
    pub target_faction: Faction,
    pub is_crit: bool,
}

pub fn apply_pending_damage(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    mut damage_events: MessageWriter<DamageEvent>,
    mut pending: MessageReader<PendingDamage>,
    mut query: Query<(&mut Health, &ComputedStats, &Transform, &Faction)>,
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

        let Ok((mut health, stats, transform, faction)) = query.get_mut(hit.target) else {
            continue;
        };

        let mut amount = hit.amount;
        let mut is_crit = false;

        if let Some(source_entity) = hit.source {
            if let Ok(source_stats) = stats_query.get(source_entity) {
                let chance = crit_chance_id.map(|id| source_stats.get(id)).unwrap_or(0.0);
                if rand::random::<f32>() < chance {
                    let multiplier = crit_multiplier_id.map(|id| source_stats.get(id)).unwrap_or(1.5);
                    amount *= multiplier;
                    is_crit = true;
                }
            }
        }

        let max = max_life_id.map(|id| stats.get(id)).unwrap_or(f32::MAX);
        health.current = (health.current - amount).clamp(0.0, max);

        damage_events.write(DamageEvent {
            position: transform.translation,
            amount,
            target_faction: *faction,
            is_crit,
        });

        if let Ok(mut entity_commands) = commands.get_entity(hit.target) {
            entity_commands.insert(HitFlash::new());
        }
    }
}
