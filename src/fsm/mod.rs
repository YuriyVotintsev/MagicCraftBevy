mod behaviour_registry;
mod components;
mod events;
mod loader;
mod registry;
mod spawn;
mod systems;
mod transition_registry;
mod types;

pub use behaviour_registry::BehaviourRegistry;
#[allow(unused_imports)]
pub use components::{CurrentState, MobType};
pub use events::StateTransition;
pub use registry::MobRegistry;
pub use spawn::spawn_mob;
pub use transition_registry::TransitionRegistry;
#[allow(unused_imports)]
pub use types::{BehaviourDef, MobDef, Shape, StateDef, TransitionDef, VisualDef};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::GameState;
use systems::fsm_transition_system;

pub struct FsmPlugin;

impl Plugin for FsmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BehaviourRegistry>()
            .init_resource::<TransitionRegistry>()
            .add_message::<StateTransition>()
            .add_systems(
                Update,
                fsm_transition_system
                    .in_set(GameSet::MobAI)
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}
