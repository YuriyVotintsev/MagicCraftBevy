use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::Player;
use crate::artifact::{Burning, Frozen, OnHitEffectStack};
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
    pub on_hit: OnHitEffectStack,
}

#[derive(Component)]
pub struct Shield {
    pub max_block: f32,
    pub current: f32,
    pub recharge: f32,
    pub recharge_cooldown: f32,
}

const SHIELD_HIT_COOLDOWN: f32 = 1.5;

pub fn register_systems(app: &mut App) {
    app.add_message::<PendingDamage>()
        .add_systems(Update, apply_pending_damage.in_set(GameSet::DamageApply))
        .add_systems(Update, tick_shield.in_set(GameSet::WaveManagement));
}

#[allow(clippy::too_many_arguments)]
pub fn apply_pending_damage(
    mut commands: Commands,
    mut pending: ResMut<bevy::ecs::message::Messages<PendingDamage>>,
    mut health_q: Query<&mut Health>,
    stats_q: Query<&ComputedStats>,
    invulnerable: Query<(), With<InvulnerableStack>>,
    player_q: Query<(), With<Player>>,
    mut shield_q: Query<&mut Shield>,
    mut velocity_q: Query<(&Transform, &mut LinearVelocity)>,
    transform_q: Query<&Transform>,
) {
    let mut to_emit: Vec<PendingDamage> = Vec::new();
    let mut to_heal: Vec<(Entity, f32)> = Vec::new();
    let drained: Vec<PendingDamage> = pending.drain().collect();

    for hit in drained {
        if invulnerable.get(hit.target).is_ok() {
            continue;
        }
        if health_q.get(hit.target).is_err() {
            continue;
        }

        let target_is_player = player_q.contains(hit.target);
        let target_dodge = stats_q
            .get(hit.target)
            .map(|s| s.final_of(Stat::DodgeChance))
            .unwrap_or(0.0);
        let target_thorns = stats_q
            .get(hit.target)
            .map(|s| s.final_of(Stat::Thorns))
            .unwrap_or(0.0);
        let target_max_life = stats_q
            .get(hit.target)
            .map(|s| s.final_of(Stat::MaxLife).max(1.0))
            .unwrap_or(1.0);

        if target_is_player && target_dodge > 0.0 && rand::random::<f32>() < target_dodge {
            if let Ok(mut ec) = commands.get_entity(hit.target) {
                ec.insert(HitFlash::new());
            }
            continue;
        }

        let mut amount = hit.amount;

        if let Ok(mut shield) = shield_q.get_mut(hit.target) {
            let absorbed = amount.min(shield.current);
            shield.current -= absorbed;
            amount -= absorbed;
            shield.recharge_cooldown = SHIELD_HIT_COOLDOWN;
        }

        if amount <= 0.0 {
            if let Ok(mut ec) = commands.get_entity(hit.target) {
                ec.insert(HitFlash::new());
            }
            continue;
        }

        if let Some(source_entity) = hit.source {
            if let Ok(source_stats) = stats_q.get(source_entity) {
                let chance = source_stats.final_of(Stat::CritChance);
                if rand::random::<f32>() < chance {
                    let multiplier = source_stats.final_of(Stat::CritMultiplier);
                    let effective = if multiplier > 0.0 { multiplier } else { 1.5 };
                    amount *= effective;
                }
            }
        }

        if let Ok(mut health) = health_q.get_mut(hit.target) {
            health.current = (health.current - amount).clamp(0.0, target_max_life);
        }

        if let Ok(mut ec) = commands.get_entity(hit.target) {
            ec.insert(HitFlash::new());
        }

        if hit.on_hit.lifesteal_pct > 0.0 {
            if let Some(src) = hit.source {
                if player_q.contains(src) {
                    to_heal.push((src, amount * hit.on_hit.lifesteal_pct));
                }
            }
        }

        if hit.on_hit.knockback > 0.0 {
            if let Some(src) = hit.source {
                if let (Ok(src_t), Ok((tgt_t, mut tgt_v))) =
                    (transform_q.get(src), velocity_q.get_mut(hit.target))
                {
                    let src_2d = crate::coord::to_2d(src_t.translation);
                    let tgt_2d = crate::coord::to_2d(tgt_t.translation);
                    let dir = (tgt_2d - src_2d).normalize_or_zero();
                    let impulse = dir * hit.on_hit.knockback;
                    tgt_v.0 += crate::coord::ground_vel(impulse);
                }
            }
        }

        if let Some((dps, duration)) = hit.on_hit.burn {
            if let Ok(mut ec) = commands.get_entity(hit.target) {
                ec.insert(Burning {
                    dps,
                    remaining: duration,
                    source: hit.source,
                });
            }
        }
        if let Some((chance, duration)) = hit.on_hit.freeze {
            if rand::random::<f32>() < chance {
                if let Ok(mut ec) = commands.get_entity(hit.target) {
                    ec.insert(Frozen {
                        remaining: duration,
                        slow_pct: 0.6,
                    });
                }
            }
        }

        if target_is_player && target_thorns > 0.0 {
            if let Some(src) = hit.source {
                to_emit.push(PendingDamage {
                    target: src,
                    amount: amount * target_thorns,
                    source: Some(hit.target),
                    on_hit: OnHitEffectStack::default(),
                });
            }
        }
    }

    for (entity, heal) in to_heal {
        if let Ok(mut health) = health_q.get_mut(entity) {
            let max = stats_q
                .get(entity)
                .map(|s| s.final_of(Stat::MaxLife).max(1.0))
                .unwrap_or(1.0);
            health.current = (health.current + heal).min(max);
        }
    }

    for ev in to_emit {
        pending.write(ev);
    }
}

fn tick_shield(time: Res<Time>, mut q: Query<&mut Shield>) {
    let dt = time.delta_secs();
    for mut shield in &mut q {
        if shield.recharge_cooldown > 0.0 {
            shield.recharge_cooldown -= dt;
            continue;
        }
        if shield.current < shield.max_block {
            shield.current = (shield.current + shield.recharge * dt).min(shield.max_block);
        }
    }
}
