pub mod context;
pub mod expr;
pub mod expr_parser;
mod blueprint_def;
mod core_components;
pub mod entity_def;
pub mod spawn;
mod registry;
mod recalc;
mod cleanup;
pub mod state;
pub mod activation;

#[macro_use]
mod macros;

pub mod components;

#[cfg(test)]
mod tests;

pub use registry::BlueprintRegistry;
pub use blueprint_def::BlueprintDefRaw;
pub use core_components::{SpawnSource, AbilityInput, BlueprintId};
pub use registry::attach_ability;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::wave::WavePhase;

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueprintRegistry>()
            .add_message::<state::StateTransition>();

        components::register_component_systems(app);

        app.add_systems(
            PreUpdate,
            recalc::recalculate_on_stats_change
                .after(crate::stats::systems::recalculate_stats)
                .run_if(not(in_state(GameState::Loading))),
        );

        app.add_systems(
            Update,
            (
                cleanup::cleanup_orphaned_abilities.in_set(crate::schedule::GameSet::Cleanup),
                state::state_transition_system.in_set(crate::schedule::GameSet::MobAI),
                activation::ability_activation_system.in_set(crate::schedule::GameSet::AbilityActivation),
            )
                .run_if(in_state(WavePhase::Combat)),
        );
    }
}
