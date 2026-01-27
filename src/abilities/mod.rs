pub mod ids;
mod context;
mod owner;
mod effect_def;
mod activator_def;
mod ability_def;
mod components;
mod registry;
mod dispatcher;

pub mod activators;
pub mod effects;

#[allow(unused_imports)]
pub use context::{AbilityContext, ContextValue};
#[allow(unused_imports)]
pub use owner::OwnedBy;
#[allow(unused_imports)]
pub use effect_def::{EffectDef, EffectDefRaw, ParamValue, ParamValueRaw};
#[allow(unused_imports)]
pub use activator_def::{ActivatorDef, ActivatorDefRaw, ActivatorState, ActivationResult};
#[allow(unused_imports)]
pub use ability_def::{AbilityDef, AbilityDefRaw};
#[allow(unused_imports)]
pub use components::{Abilities, AbilityInstance, AbilityInput};
#[allow(unused_imports)]
pub use registry::{Activator, EffectExecutor, ActivatorRegistry, EffectRegistry, AbilityRegistry};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::GameState;

mod projectile_systems;

#[allow(unused_imports)]
pub use effects::Projectile;

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        let mut activator_registry = ActivatorRegistry::new();
        activators::register_activators(&mut activator_registry);

        let mut effect_registry = EffectRegistry::new();
        effects::register_effects(&mut effect_registry);

        app.insert_resource(activator_registry)
            .insert_resource(effect_registry)
            .init_resource::<AbilityRegistry>()
            .add_systems(
                Update,
                dispatcher::ability_dispatcher
                    .in_set(GameSet::AbilityActivation)
                    .run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                projectile_systems::projectile_collision
                    .in_set(GameSet::AbilityExecution)
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}
