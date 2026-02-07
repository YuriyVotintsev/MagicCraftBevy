pub mod ids;
pub mod context;
pub mod expr;
pub mod expr_parser;
pub mod node;
mod ability_def;
mod core_components;
pub mod entity_def;
pub mod spawn;
mod cleanup;
pub mod state;
pub mod activation;

#[macro_use]
mod macros;

pub mod components;

pub use node::AbilityRegistry;
pub use ability_def::{AbilityDef, AbilityDefRaw};
pub use core_components::{AbilitySource, AbilityInput};
pub use node::attach_ability;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::stats::ComputedStats;
use crate::wave::WavePhase;
use spawn::StoredComponentDefs;

fn recalculate_on_stats_change(
    mut commands: Commands,
    query: Query<(Entity, &AbilitySource, &ComputedStats, &StoredComponentDefs), Changed<ComputedStats>>,
) {
    for (entity, source, stats, defs) in &query {
        for def in defs.all() {
            def.update_component(&mut commands.entity(entity), source, stats);
        }
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityRegistry>()
            .add_message::<state::StateTransition>();

        components::register_component_systems(app);

        app.add_systems(
            PreUpdate,
            recalculate_on_stats_change
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
