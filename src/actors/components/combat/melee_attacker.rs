use avian3d::prelude::CollidingEntities;
use bevy::prelude::*;

use super::shot_fired::ShotFired;
use super::PendingDamage;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::Faction;

const MELEE_STRIKE_DAMAGE_PCT: f32 = 1.0;

#[derive(Component)]
pub struct MeleeAttacker {
    pub cooldown: f32,
    pub elapsed: f32,
}

impl MeleeAttacker {
    pub fn new(cooldown: f32) -> Self {
        Self { cooldown, elapsed: 0.0 }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, melee_attacker_system.in_set(GameSet::MobAI));
}

fn melee_attacker_system(
    mut commands: Commands,
    mut pending: MessageWriter<PendingDamage>,
    time: Res<Time>,
    stats_query: Query<&ComputedStats>,
    mut attackers: Query<
        (Entity, &mut MeleeAttacker, &Faction, &CollidingEntities),
        Without<crate::wave::RiseFromGround>,
    >,
    faction_query: Query<&Faction>,
) {
    for (caster, mut attacker, caster_faction, colliding) in &mut attackers {
        attacker.elapsed += time.delta_secs();
        if attacker.elapsed < attacker.cooldown {
            continue;
        }

        let target = colliding.iter().copied().find(|t| {
            faction_query
                .get(*t)
                .map(|f| f != caster_faction)
                .unwrap_or(false)
        });
        let Some(target) = target else { continue };

        attacker.elapsed = 0.0;
        let damage = stats_query
            .get(caster)
            .map(|s| s.final_of(Stat::PhysicalDamage) * MELEE_STRIKE_DAMAGE_PCT)
            .unwrap_or(0.0);

        pending.write(PendingDamage {
            target,
            amount: damage,
            source: Some(caster),
        });
        commands.entity(caster).insert(ShotFired);
    }
}
