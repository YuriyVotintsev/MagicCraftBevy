use bevy::prelude::*;

use super::melee_strike::MeleeStrike;
use super::shot_fired::ShotFired;
use super::Caster;
use crate::actors::Player;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::Faction;

pub const MELEE_STRIKE_RANGE: f32 = 300.0;
const MELEE_STRIKE_DAMAGE_PCT: f32 = 1.0;

#[derive(Component)]
pub struct MeleeAttacker {
    pub cooldown: f32,
    pub trigger_range: f32,
    pub elapsed: f32,
}

impl MeleeAttacker {
    pub fn new(cooldown: f32, trigger_range: f32) -> Self {
        Self { cooldown, trigger_range, elapsed: 0.0 }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        melee_attacker_system.in_set(GameSet::MobAI),
    );
}

fn melee_attacker_system(
    mut commands: Commands,
    time: Res<Time>,
    stats_query: Query<&ComputedStats>,
    mut query: Query<(Entity, &Transform, &mut MeleeAttacker, &Faction), Without<crate::wave::RiseFromGround>>,
    player: Option<Single<&Transform, (With<Player>, Without<MeleeAttacker>)>>,
) {
    let Some(player) = player else { return };
    for (caster, transform, mut attacker, faction) in &mut query {
        attacker.elapsed += time.delta_secs();
        if attacker.elapsed < attacker.cooldown { continue }

        if transform.translation.distance(player.translation) > attacker.trigger_range {
            continue;
        }

        attacker.elapsed = 0.0;

        let damage = stats_query
            .get(caster)
            .map(|s| s.final_of(Stat::PhysicalDamage) * MELEE_STRIKE_DAMAGE_PCT)
            .unwrap_or(0.0);

        commands.entity(caster).insert(ShotFired);
        commands.spawn((
            MeleeStrike { range: MELEE_STRIKE_RANGE, damage },
            Caster(caster),
            *faction,
        ));
    }
}
