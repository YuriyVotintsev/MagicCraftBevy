pub mod ids;
pub mod context;
pub mod expr;
pub mod expr_parser;
pub mod eval_context;
pub mod node;
mod ability_def;
mod activator_support;
mod core_components;
pub mod entity_def;
pub mod spawn;
mod cleanup;

#[macro_use]
mod macros;

pub mod components;

pub use context::TargetInfo;
pub use node::AbilityRegistry;
pub use ability_def::{AbilityDef, AbilityDefRaw};
pub use core_components::{AbilityInputs, InputState, AbilitySource};
pub use node::attach_ability;

use bevy::prelude::*;

use crate::wave::WavePhase;

fn clear_ability_inputs(mut query: Query<&mut AbilityInputs>) {
    for mut inputs in &mut query {
        inputs.clear();
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbilityRegistry>();

        components::register_component_systems(app);

        app.add_systems(
            Update,
            (
                clear_ability_inputs.before(crate::schedule::GameSet::Input),
                cleanup::cleanup_orphaned_abilities,
            )
                .run_if(in_state(WavePhase::Combat)),
        );

        app.add_systems(OnExit(WavePhase::Combat), clear_ability_inputs);
    }
}
