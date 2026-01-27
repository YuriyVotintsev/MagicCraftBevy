mod components;
mod events;
mod loader;
mod registry;
mod spawn;
mod systems;
mod types;

#[allow(unused_imports)]
pub use components::{CurrentState, MobType};
pub use events::StateTransition;
pub use registry::MobRegistry;
pub use spawn::spawn_mob;
#[allow(unused_imports)]
pub use types::{BehaviourDef, ColliderDef, ColliderShape, MobDef, Shape, StateDef, TransitionDef, VisualDef};

use bevy::prelude::*;

use crate::mob_ai::{
    after_time_system, keep_distance_system, move_toward_player_system, use_abilities_system,
    when_near_system,
};
use crate::schedule::GameSet;
use crate::GameState;
use systems::fsm_transition_system;

pub struct FsmPlugin;

impl Plugin for FsmPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StateTransition>()
            .add_systems(
                Update,
                (
                    move_toward_player_system,
                    keep_distance_system,
                    use_abilities_system,
                    when_near_system,
                    after_time_system,
                    fsm_transition_system,
                )
                    .chain()
                    .in_set(GameSet::MobAI)
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}
