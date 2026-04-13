use bevy::prelude::*;

use crate::actors::components::ability::melee_strike::MeleeStrike;
use crate::actors::SpawnSource;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::Faction;

use super::ShotFired;

pub const MELEE_STRIKE_RANGE: f32 = 300.0;

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
    transforms: Query<&Transform, Without<MeleeAttacker>>,
    stats_query: Query<&ComputedStats>,
    mut query: Query<(Entity, &Transform, &mut MeleeAttacker, &SpawnSource, &Faction), Without<crate::wave::summoning::RiseFromGround>>,
) {
    for (caster, transform, mut attacker, source, faction) in &mut query {
        attacker.elapsed += time.delta_secs();
        if attacker.elapsed < attacker.cooldown { continue }

        let Some(target_entity) = source.target.entity else { continue };
        let Ok(target_transform) = transforms.get(target_entity) else { continue };

        if transform.translation.distance(target_transform.translation) > attacker.trigger_range {
            continue;
        }

        attacker.elapsed = 0.0;

        let caster_pos = crate::coord::to_2d(transform.translation);
        let target_pos = crate::coord::to_2d(target_transform.translation);
        let direction = (target_pos - caster_pos).normalize_or_zero();
        let damage = stats_query.get(caster).map(|s| s.get(Stat::PhysicalDamageFlat)).unwrap_or(0.0);

        commands.entity(caster).insert(ShotFired);
        commands.spawn((
            MeleeStrike { range: MELEE_STRIKE_RANGE, damage },
            SpawnSource::with_target(caster, caster_pos, crate::actors::TargetInfo {
                entity: Some(target_entity),
                position: Some(target_pos),
                direction: Some(direction),
            }),
            *faction,
        ));
    }
}
