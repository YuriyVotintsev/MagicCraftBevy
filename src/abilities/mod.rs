pub mod ids;
mod context;
mod owner;
mod effect_def;
mod activator_def;
mod ability_def;
mod components;
mod registry;
mod spawn_helpers;

#[macro_use]
mod macros;

pub mod activators;
pub mod effects;

#[allow(unused_imports)]
pub use context::{AbilityContext, ContextValue};
#[allow(unused_imports)]
pub use owner::OwnedBy;
#[allow(unused_imports)]
pub use effect_def::{EffectDef, EffectDefRaw, ParamValue, ParamValueRaw};
#[allow(unused_imports)]
pub use activator_def::{ActivatorDef, ActivatorDefRaw};
#[allow(unused_imports)]
pub use ability_def::{AbilityDef, AbilityDefRaw};
#[allow(unused_imports)]
pub use components::{AbilityInputs, AbilityId, InputState};
#[allow(unused_imports)]
pub use registry::{EffectHandler, ActivatorHandler, ActivatorRegistry, EffectRegistry, AbilityRegistry};
#[allow(unused_imports)]
pub use spawn_helpers::add_ability_activator;
#[allow(unused_imports)]
pub use activators::{
    OnInputActivations, PassiveActivations, WhileHeldActivations, IntervalActivations,
};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::wave::WavePhase;

#[allow(unused_imports)]
pub use effects::Projectile;

fn clear_ability_inputs(mut query: Query<&mut AbilityInputs>) {
    for mut inputs in &mut query {
        inputs.clear();
    }
}

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        let mut activator_registry = ActivatorRegistry::new();
        activators::register_all(app, &mut activator_registry);

        let mut effect_registry = EffectRegistry::new();
        effects::register_all(app, &mut effect_registry);

        app.insert_resource(activator_registry)
            .insert_resource(effect_registry)
            .init_resource::<AbilityRegistry>();

        app.add_systems(
            Update,
            clear_ability_inputs
                .before(GameSet::Input)
                .run_if(in_state(WavePhase::Combat)),
        );
    }
}
