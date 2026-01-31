pub mod ids;
mod context;
mod param;
mod trigger_def;
mod ability_def;
mod components;
mod registry;
mod spawn_helpers;
pub mod events;
mod dispatcher;
mod lifecycle;

#[macro_use]
mod macros;

pub mod triggers;
pub mod actions;

#[allow(unused_imports)]
pub use context::{AbilityContext, ContextValue};
#[allow(unused_imports)]
pub use param::{ParamValue, ParamValueRaw};
#[allow(unused_imports)]
pub use trigger_def::{TriggerDef, TriggerDefRaw, ActionDef, ActionDefRaw};
#[allow(unused_imports)]
pub use ability_def::{AbilityDef, AbilityDefRaw};
#[allow(unused_imports)]
pub use components::{AbilityInputs, AbilityId, InputState, AbilitySource};
#[allow(unused_imports)]
pub use registry::{TriggerHandler, ActionHandler, TriggerRegistry, ActionRegistry, AbilityRegistry};
#[allow(unused_imports)]
pub use spawn_helpers::add_ability_trigger;
#[allow(unused_imports)]
pub use lifecycle::AttachedTo;
#[allow(unused_imports)]
pub use triggers::{
    on_input::OnInputTriggers, every_frame::EveryFrameTriggers,
    while_held::WhileHeldTriggers, interval::IntervalTriggers,
};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::wave::WavePhase;
use crate::game_state::GameState;

#[allow(unused_imports)]
pub use actions::spawn_projectile::Projectile;
#[allow(unused_imports)]
pub use events::{TriggerAbilityEvent, ExecuteActionEvent};

fn clear_ability_inputs(mut query: Query<&mut AbilityInputs>) {
    for mut inputs in &mut query {
        inputs.clear();
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Messages<TriggerAbilityEvent>>();
        app.init_resource::<Messages<ExecuteActionEvent>>();

        let mut trigger_registry = TriggerRegistry::new();
        triggers::register_all(app, &mut trigger_registry);

        let mut action_registry = ActionRegistry::new();
        actions::register_all(app, &mut action_registry);

        app.insert_resource(trigger_registry)
            .insert_resource(action_registry)
            .init_resource::<AbilityRegistry>();

        app.add_systems(
            Update,
            dispatcher::ability_dispatcher
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
