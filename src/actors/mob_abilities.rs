use bevy::prelude::*;

use crate::actors::abilities::{fire_ability, AbilitiesBalance, AbilityKind};
use crate::actors::target_info::TargetInfo;
use crate::actors::SpawnSource;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;

#[derive(Component)]
pub struct ShotFired;

#[derive(Component)]
pub struct MobAbilities {
    pub abilities: Vec<AbilityKind>,
    pub cooldown: f32,
    pub max_range: Option<f32>,
    pub elapsed: f32,
}

impl MobAbilities {
    pub fn new(abilities: Vec<AbilityKind>, cooldown: f32, max_range: Option<f32>) -> Self {
        Self { abilities, cooldown, max_range, elapsed: 0.0 }
    }
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (cleanup_shot_fired, mob_abilities_system).chain().in_set(GameSet::MobAI),
    );
}

fn cleanup_shot_fired(mut commands: Commands, q: Query<Entity, With<ShotFired>>) {
    for e in &q { commands.entity(e).remove::<ShotFired>(); }
}

fn mob_abilities_system(
    mut commands: Commands,
    time: Res<Time>,
    abilities_balance: Res<AbilitiesBalance>,
    transforms: Query<&Transform, Without<MobAbilities>>,
    stats_query: Query<&ComputedStats>,
    mut query: Query<(Entity, &Transform, &mut MobAbilities, &SpawnSource), Without<crate::summoning::RiseFromGround>>,
) {
    for (caster, transform, mut ma, source) in &mut query {
        ma.elapsed += time.delta_secs();
        if ma.elapsed < ma.cooldown { continue }

        let Some(target_entity) = source.target.entity else { continue };
        let Ok(target_transform) = transforms.get(target_entity) else { continue };

        if let Some(max_range) = ma.max_range {
            if transform.translation.distance(target_transform.translation) > max_range {
                continue;
            }
        }

        ma.elapsed = 0.0;

        let caster_pos = crate::coord::to_2d(transform.translation);
        let target_pos = crate::coord::to_2d(target_transform.translation);
        let direction = (target_pos - caster_pos).normalize_or_zero();
        let target_info = TargetInfo {
            entity: Some(target_entity),
            position: Some(target_pos),
            direction: Some(direction),
        };
        let caster_stats = stats_query.get(caster).ok();

        commands.entity(caster).insert(ShotFired);
        for &kind in &ma.abilities {
            fire_ability(
                &mut commands,
                kind,
                caster,
                caster_pos,
                source.caster_faction,
                target_info,
                &abilities_balance,
                caster_stats,
            );
        }
    }
}
