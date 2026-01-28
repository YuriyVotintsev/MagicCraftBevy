pub mod ids;
mod context;
mod owner;
mod effect_def;
mod activator_def;
mod ability_def;
mod components;
mod registry;
mod spawn_helpers;

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
pub use registry::{EffectExecutor, ActivatorRegistry, EffectRegistry, AbilityRegistry};
#[allow(unused_imports)]
pub use spawn_helpers::add_ability_activator;
#[allow(unused_imports)]
pub use activators::{
    OnInputActivations, PassiveActivations, WhileHeldActivations, IntervalActivations,
};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::wave::WavePhase;

mod projectile_systems;
mod orbiting_systems;
mod dash_systems;
mod meteor_systems;
mod shield_systems;

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
        activator_registry.register_name("on_input");
        activator_registry.register_name("passive");
        activator_registry.register_name("while_held");
        activator_registry.register_name("interval");

        let mut effect_registry = EffectRegistry::new();
        effects::register_effects(&mut effect_registry);

        app.insert_resource(activator_registry)
            .insert_resource(effect_registry)
            .init_resource::<AbilityRegistry>();

        activators::register_activator_systems(app);

        app.add_systems(
            Update,
            clear_ability_inputs
                .before(GameSet::Input)
                .run_if(in_state(WavePhase::Combat)),
        )
        .add_systems(
            Update,
            (
                projectile_systems::projectile_collision,
                orbiting_systems::update_orbiting_positions,
                dash_systems::update_dashing,
                meteor_systems::meteor_target_finder,
                meteor_systems::meteor_falling_update,
                meteor_systems::meteor_explosion_damage,
                shield_systems::update_shield,
                shield_systems::update_shield_visual,
            )
                .in_set(GameSet::AbilityExecution),
        )
        .add_systems(
            PostUpdate,
            orbiting_systems::cleanup_orbiting_on_owner_despawn
                .run_if(in_state(crate::GameState::Playing)),
        );
    }
}
