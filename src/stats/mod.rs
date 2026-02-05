mod calculators;
mod computed_stats;
mod dirty_stats;
mod expression;
mod health;
pub mod loader;
mod modifiers;
mod pending_damage;
mod stat_id;
mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::{ComputedStats, DEFAULT_STATS};
pub use dirty_stats::DirtyStats;
pub use expression::Expression;
pub use health::Dead;
pub use health::{death_system, DeathEvent, Health};
pub use modifiers::Modifiers;
pub use pending_damage::PendingDamage;

use crate::schedule::{GameSet, PostGameSet};
use crate::Faction;
use crate::GameState;

#[derive(Message)]
pub struct DamageEvent {
    pub position: Vec3,
    pub amount: f32,
    pub target_faction: Faction,
}
pub use stat_id::{AggregationType, StatId, StatRegistry};

use bevy::prelude::*;

use crate::wave::InvulnerableStack;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DeathEvent>()
            .add_message::<DamageEvent>()
            .add_systems(
                PreUpdate,
                (systems::mark_dirty_on_modifier_change, systems::recalculate_stats)
                    .chain()
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                (discard_damage_on_invulnerable, apply_pending_damage).in_set(GameSet::Damage),
            )
            .add_systems(PostUpdate, health::death_system.in_set(PostGameSet))
            .add_systems(
                Last,
                health::cleanup_dead.run_if(not(in_state(GameState::Loading))),
            );
    }
}

fn discard_damage_on_invulnerable(
    mut commands: Commands,
    query: Query<Entity, (With<PendingDamage>, With<InvulnerableStack>)>,
) {
    for entity in &query {
        commands.entity(entity).remove::<PendingDamage>();
    }
}

fn apply_pending_damage(
    mut commands: Commands,
    stat_registry: Res<StatRegistry>,
    mut damage_events: MessageWriter<DamageEvent>,
    mut query: Query<
        (Entity, &PendingDamage, &mut Health, &ComputedStats, &Transform, &Faction),
        Without<InvulnerableStack>,
    >,
) {
    let max_life_id = stat_registry.get("max_life");

    for (entity, pending, mut health, stats, transform, faction) in &mut query {
        let max = max_life_id.map(|id| stats.get(id)).unwrap_or(f32::MAX);
        health.current = (health.current - pending.0).clamp(0.0, max);

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

