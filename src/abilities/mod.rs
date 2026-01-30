pub mod ids;
mod context;
mod owner;
mod effect_def;
mod trigger_def;
mod ability_def;
mod components;
mod registry;
mod spawn_helpers;
pub mod events;
mod dispatcher;

#[macro_use]
mod macros;

pub mod triggers;
pub mod effects;

#[allow(unused_imports)]
pub use context::{AbilityContext, ContextValue};
#[allow(unused_imports)]
pub use owner::OwnedBy;
#[allow(unused_imports)]
pub use effect_def::{EffectDef, EffectDefRaw, ParamValue, ParamValueRaw};
#[allow(unused_imports)]
pub use trigger_def::{TriggerDef, TriggerDefRaw};
#[allow(unused_imports)]
pub use ability_def::{AbilityDef, AbilityDefRaw};
#[allow(unused_imports)]
pub use components::{AbilityInputs, AbilityId, InputState};
#[allow(unused_imports)]
pub use registry::{EffectHandler, TriggerHandler, TriggerRegistry, EffectRegistry, AbilityRegistry};
#[allow(unused_imports)]
pub use spawn_helpers::add_ability_trigger;
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
pub use effects::spawn_projectile::Projectile;
#[allow(unused_imports)]
pub use events::{TriggerAbilityEvent, ExecuteEffectEvent};

fn clear_ability_inputs(mut query: Query<&mut AbilityInputs>) {
    for mut inputs in &mut query {
        inputs.clear();
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Messages<TriggerAbilityEvent>>();
        app.init_resource::<Messages<ExecuteEffectEvent>>();

        let mut trigger_registry = TriggerRegistry::new();
        triggers::register_all(app, &mut trigger_registry);

        let mut effect_registry = EffectRegistry::new();
        effects::register_all(app, &mut effect_registry);

        app.insert_resource(trigger_registry)
            .insert_resource(effect_registry)
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
    }
}
