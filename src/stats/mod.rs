mod calculators;
mod computed_stats;
mod damage;
pub mod display;
mod dirty_stats;
mod health;
mod modifiers;
mod pending_damage;
mod registry;
pub(crate) mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::ComputedStats;
pub use dirty_stats::DirtyStats;
pub use display::{FormatSpan, SignMode, StatDisplayRegistry, ValueTemplate};
pub use health::{Dead, SkipCleanup, death_system, DeathEvent};
pub use modifiers::Modifiers;
pub use pending_damage::PendingDamage;
pub use registry::Stat;

use bevy::prelude::*;

use crate::schedule::{GameSet, PostGameSet};
use crate::GameState;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PendingDamage>()
            .add_message::<DeathEvent>()
            .add_systems(
                PreUpdate,
                (systems::mark_dirty_on_modifier_change, systems::recalculate_stats)
                    .chain()
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                damage::apply_pending_damage.in_set(GameSet::DamageApply),
            )
            .add_systems(PostUpdate, health::death_system.in_set(PostGameSet))
            .add_systems(
                Last,
                health::cleanup_dead.run_if(not(in_state(GameState::Loading))),
            );
    }
}

pub fn build_stat_system() -> (StatCalculators, StatDisplayRegistry) {
    (StatCalculators::build(), StatDisplayRegistry::build())
}
