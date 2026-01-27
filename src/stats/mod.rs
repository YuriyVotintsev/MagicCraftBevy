mod calculators;
mod computed_stats;
mod dirty_stats;
mod health;
mod loader;
mod modifiers;
mod pending_damage;
mod stat_id;
mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::ComputedStats;
pub use dirty_stats::DirtyStats;
pub use health::{DeathEvent, Health};
pub use loader::load_stats;
#[allow(unused_imports)]
pub use modifiers::{Modifier, Modifiers};
pub use pending_damage::PendingDamage;

use crate::Faction;

#[derive(Message)]
pub struct DamageEvent {
    pub position: Vec3,
    pub amount: f32,
    pub target_faction: Faction,
}
#[allow(unused_imports)]
pub use stat_id::{AggregationType, StatDef, StatId, StatRegistry};

use bevy::prelude::*;

use crate::wave::Invulnerable;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        let (registry, calculators) = load_stats(
            "assets/stats/stat_ids.ron",
            "assets/stats/calculators.ron",
        );

        app.insert_resource(registry)
            .insert_resource(calculators)
            .add_message::<DeathEvent>()
            .add_message::<DamageEvent>()
            .add_systems(PreUpdate, systems::recalculate_stats)
            .add_systems(Update, apply_pending_damage)
            .add_systems(
                PostUpdate,
                (
                    health::sync_health_to_max_life,
                    health::death_system,
                    health::handle_player_death,
                )
                    .chain(),
            );
    }
}

fn apply_pending_damage(
    mut commands: Commands,
    mut damage_events: MessageWriter<DamageEvent>,
    mut query: Query<(Entity, &PendingDamage, &mut Health, &Transform, &Faction), Without<Invulnerable>>,
) {
    for (entity, pending, mut health, transform, faction) in &mut query {
        health.take_damage(pending.0);

        damage_events.write(DamageEvent {
            position: transform.translation,
            amount: pending.0,
            target_faction: *faction,
        });

        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<PendingDamage>();
        }
    }
}

#[derive(Bundle, Default)]
#[allow(dead_code)]
pub struct StatsBundle {
    pub modifiers: Modifiers,
    pub computed: ComputedStats,
    pub dirty: DirtyStats,
}
