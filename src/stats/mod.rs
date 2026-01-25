mod calculators;
mod computed_stats;
mod dirty_stats;
mod health;
mod loader;
mod modifiers;
mod stat_id;
mod systems;

pub use calculators::StatCalculators;
pub use computed_stats::ComputedStats;
pub use dirty_stats::DirtyStats;
pub use health::Health;
pub use loader::load_stats;
pub use modifiers::{Modifier, Modifiers};
pub use stat_id::{AggregationType, StatDef, StatId, StatRegistry};

use bevy::prelude::*;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        let (registry, calculators) = load_stats(
            "assets/stats/stat_ids.ron",
            "assets/stats/calculators.ron",
        );

        app.insert_resource(registry)
            .insert_resource(calculators)
            .add_systems(PreUpdate, systems::recalculate_stats)
            .add_systems(
                PostUpdate,
                (health::sync_health_to_max_life, health::death_system).chain(),
            );
    }
}

#[derive(Bundle, Default)]
pub struct StatsBundle {
    pub modifiers: Modifiers,
    pub computed: ComputedStats,
    pub dirty: DirtyStats,
}
