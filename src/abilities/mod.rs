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

#[allow(unused_imports)]
pub use context::{AbilityContext, ContextValue};
#[allow(unused_imports)]
pub use param::{ParamValue, ParamValueRaw};
#[allow(unused_imports)]
pub use node::{NodeDef, NodeDefRaw, NodeKind, NodeHandler, NodeRegistry, AbilityRegistry};
#[allow(unused_imports)]
pub use ability_def::{AbilityDef, AbilityDefRaw};
#[allow(unused_imports)]
pub use components::{AbilityInputs, AbilityId, InputState, AbilitySource, HasOnHitTrigger};
#[allow(unused_imports)]
pub use ids::{NodeDefId, NodeTypeId};
#[allow(unused_imports)]
pub use spawn_helpers::add_ability_trigger;
#[allow(unused_imports)]
pub use lifecycle::AttachedTo;

pub use crate::building_blocks::triggers::{
    on_input::OnInputTriggers, every_frame::EveryFrameTriggers,
    while_held::WhileHeldTriggers, interval::IntervalTriggers,
};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::wave::WavePhase;
use crate::game_state::GameState;

#[allow(unused_imports)]
pub use crate::building_blocks::actions::spawn_projectile::Projectile;
#[allow(unused_imports)]
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
