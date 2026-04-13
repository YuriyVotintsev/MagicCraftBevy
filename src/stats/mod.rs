mod calculators;
mod computed_stats;
pub mod display;
mod dirty_stats;
mod modifiers;
mod registry;
pub(crate) mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::ComputedStats;
pub use dirty_stats::DirtyStats;
pub use display::{FormatSpan, SignMode, StatDisplayRegistry, ValueTemplate};
pub use modifiers::Modifiers;
pub use registry::Stat;

use bevy::prelude::*;

use crate::GameState;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (systems::mark_dirty_on_modifier_change, systems::recalculate_stats)
                .chain()
                .run_if(not(in_state(GameState::Loading))),
        );
    }
}

pub fn build_stat_system() -> (StatCalculators, StatDisplayRegistry) {
    (StatCalculators::build(), StatDisplayRegistry::build())
}
