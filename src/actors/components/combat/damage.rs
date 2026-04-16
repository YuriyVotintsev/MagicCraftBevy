use bevy::prelude::*;

use crate::hit_flash::HitFlash;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::wave::InvulnerableStack;

use super::Health;

#[derive(Message)]
pub struct PendingDamage {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}

pub fn register_systems(app: &mut App) {
    app.add_message::<PendingDamage>()
        .add_systems(Update, apply_pending_damage.in_set(GameSet::DamageApply));
}

pub fn apply_pending_damage(
    mut commands: Commands,
    mut pending: MessageReader<PendingDamage>,
    mut query: Query<(&mut Health, &ComputedStats)>,
    stats_query: Query<&ComputedStats>,
    invulnerable: Query<(), With<InvulnerableStack>>,
) {
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
                let chance = source_stats.final_of(Stat::CritChance);
                if rand::random::<f32>() < chance {
                    let multiplier = source_stats.final_of(Stat::CritMultiplier);
                    let effective = if multiplier > 0.0 { multiplier } else { 1.5 };
                    amount *= effective;
                }
            }
        }

        let max = stats.final_of(Stat::MaxLife).max(1.0);
        health.current = (health.current - amount).clamp(0.0, max);

        if let Ok(mut entity_commands) = commands.get_entity(hit.target) {
            entity_commands.insert(HitFlash::new());
        }
    }
}
