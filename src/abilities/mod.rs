pub mod ids;
pub mod context;
pub mod expr;
pub mod expr_parser;
pub mod node;
mod ability_def;
mod activator_support;
mod core_components;
pub mod entity_def;
pub mod spawn;
mod cleanup;
pub mod state;

#[macro_use]
mod macros;

pub mod components;

pub use context::TargetInfo;
pub use node::AbilityRegistry;
pub use ability_def::{AbilityDef, AbilityDefRaw};
pub use core_components::{AbilityInputs, InputState, AbilitySource};
pub use node::attach_ability;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::stats::ComputedStats;
use crate::wave::WavePhase;
use spawn::StoredComponentDefs;

fn clear_ability_inputs(mut query: Query<&mut AbilityInputs>) {
    for mut inputs in &mut query {
        inputs.clear();
    }
}

fn recalculate_on_stats_change(world: &mut World) {
    let mut to_update: Vec<(Entity, AbilitySource, ComputedStats, StoredComponentDefs)> = Vec::new();

    let mut query = world.query_filtered::<(Entity, &AbilitySource, &ComputedStats, &StoredComponentDefs), Changed<ComputedStats>>();
    for (entity, source, stats, defs) in query.iter(world) {
        to_update.push((entity, *source, stats.clone(), StoredComponentDefs {
            base: defs.base.clone(),
            state: defs.state.clone(),
        }));
    }

    for (entity, source, stats, defs) in &to_update {
        for def in defs.all() {
            def.update_component(*entity, source, stats, world);
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
                .run_if(not(in_state(GameState::Loading))),
        );

        app.add_systems(
            Update,
            (
                clear_ability_inputs.before(crate::schedule::GameSet::Input),
                cleanup::cleanup_orphaned_abilities,
                state::state_transition_system.in_set(crate::schedule::GameSet::MobAI),
            )
                .run_if(in_state(WavePhase::Combat)),
        );

        app.add_systems(OnExit(WavePhase::Combat), clear_ability_inputs);
    }
}
