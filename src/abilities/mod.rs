pub mod ids;
pub mod context;
pub mod param;
pub mod node;
mod ability_def;
mod components;
mod spawn_helpers;
pub mod events;
mod dispatcher;
mod lifecycle;

#[macro_use]
mod macros;

pub use crate::building_blocks::triggers;
pub use crate::building_blocks::actions;

pub use context::AbilityContext;
pub use param::{ParamValue, ParamValueRaw};
pub use node::{NodeDef, NodeDefRaw, NodeKind, NodeRegistry, AbilityRegistry};
pub use ability_def::{AbilityDef, AbilityDefRaw};
pub use components::{AbilityInputs, InputState, AbilitySource};
pub use ids::NodeDefId;
pub use spawn_helpers::add_ability_trigger;
pub use lifecycle::AttachedTo;

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::wave::WavePhase;
use crate::game_state::GameState;

pub use events::{TriggerAbilityEvent, ExecuteNodeEvent, NodeTriggerEvent};

fn clear_ability_inputs(mut query: Query<&mut AbilityInputs>) {
    for mut inputs in &mut query {
        inputs.clear();
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Messages<TriggerAbilityEvent>>();
        app.init_resource::<Messages<ExecuteNodeEvent>>();
        app.init_resource::<Messages<NodeTriggerEvent>>();

        let mut node_registry = NodeRegistry::new();
        triggers::register_all(app, &mut node_registry);
        actions::register_all(app, &mut node_registry);

        app.insert_resource(node_registry)
            .init_resource::<AbilityRegistry>();

        app.add_systems(
            Update,
            (dispatcher::node_ability_dispatcher, dispatcher::node_trigger_dispatcher)
                .in_set(GameSet::AbilityActivation)
                .run_if(in_state(GameState::Playing)),
        );

        app.add_systems(
            Update,
            clear_ability_inputs
                .before(GameSet::Input)
                .run_if(in_state(WavePhase::Combat)),
        );

        app.add_systems(OnExit(WavePhase::Combat), clear_ability_inputs);

        lifecycle::register_lifecycle_systems(app);
    }
}
