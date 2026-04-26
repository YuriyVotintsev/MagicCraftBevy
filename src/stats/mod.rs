mod calculators;
mod computed_stats;
mod dirty_stats;
mod modifiers;
mod registry;
mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::ComputedStats;
pub use dirty_stats::DirtyStats;
pub use modifiers::Modifiers;
pub use registry::{ModifierKind, Stat};

use bevy::prelude::*;

use crate::GameState;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StatCalculators::build())
            .add_systems(
                PreUpdate,
                (systems::mark_dirty_on_modifier_change, systems::recalculate_stats)
                    .chain()
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}
