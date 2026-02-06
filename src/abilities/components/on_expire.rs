use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::schedule::GameSet;
use super::lifetime::Lifetime;
use crate::stats::{ComputedStats, DEFAULT_STATS};

#[ability_component(SOURCE_POSITION)]
pub struct OnExpire {
    pub entities: Vec<EntityDef>,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Update, on_expire_trigger_system.in_set(GameSet::AbilityExecution));
}

fn on_expire_trigger_system(
    mut commands: Commands,
    query: Query<(Entity, &OnExpire, &AbilitySource, &Transform, &Lifetime)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform>,
) {
    for (entity, trigger, source, transform, lifetime) in &query {
        if lifetime.remaining > 0.0 {
            continue;
        }

        let caster_stats = stats_query
            .get(source.caster)
            .unwrap_or(&DEFAULT_STATS);

        let caster_pos = transforms.get(source.caster)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let source_pos = transform.translation.truncate();
        let source_info = TargetInfo::from_position(source_pos);

        let spawn_ctx = SpawnContext {
            ability_id: source.ability_id,
            caster: source.caster,
            caster_position: caster_pos,
            caster_faction: source.caster_faction,
            source: source_info,
            target: TargetInfo::EMPTY,
            stats: caster_stats,
            index: 0,
            count: 1,
        };

        for entity_def in &trigger.entities {
            crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx, None, None, None);
        }

        commands.entity(entity).remove::<OnExpire>();
    }
}
