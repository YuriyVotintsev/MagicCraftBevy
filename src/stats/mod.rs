mod calculators;
mod computed_stats;
mod dirty_stats;
mod modifiers;
mod raw_stats;
mod stat_id;
mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::ComputedStats;
pub use dirty_stats::DirtyStats;
pub use modifiers::Modifiers;
pub use raw_stats::RawStats;
pub use stat_id::StatId;

use bevy::prelude::*;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatCalculators>()
            .register_type::<StatId>()
            .register_type::<RawStats>()
            .register_type::<ComputedStats>()
            .add_systems(PreUpdate, systems::recalculate_stats);
    }
}

#[derive(Bundle, Default)]
pub struct StatsBundle {
    pub raw: RawStats,
    pub computed: ComputedStats,
    pub dirty: DirtyStats,
}
