mod ids;
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

pub use ids::*;
pub use context::{AbilityContext, ContextValue};
pub use owner::OwnedBy;
pub use effect_def::{EffectDef, EffectDefRaw, ParamValue, ParamValueRaw};
pub use activator_def::{ActivatorDef, ActivatorDefRaw, ActivatorState, ActivationResult};
pub use ability_def::{AbilityDef, AbilityDefRaw};
pub use components::{Abilities, AbilityInstance, AbilityInput};
pub use registry::{Activator, EffectExecutor, ActivatorRegistry, EffectRegistry, AbilityRegistry};

use bevy::prelude::*;

mod projectile_systems;

pub use effects::{Projectile, ProjectileVelocity};

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
            .add_systems(Update, (
                dispatcher::ability_dispatcher,
                projectile_systems::move_projectiles,
                projectile_systems::projectile_collision,
            ));
    }
}
