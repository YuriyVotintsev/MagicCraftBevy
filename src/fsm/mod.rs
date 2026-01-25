mod components;
mod events;
mod loader;
mod registry;
mod spawn;
mod systems;
mod types;

pub use components::{Collider, CurrentState, MobType};
pub use events::StateTransition;
pub use registry::MobRegistry;
pub use spawn::spawn_mob;
pub use types::{BehaviourDef, ColliderShape, MobDef, Shape, StateDef, TransitionDef, VisualDef};

use bevy::prelude::*;

use crate::mob_ai::{
    after_time_system, move_toward_player_system, use_abilities_system, when_near_system,
};
use loader::load_mobs;
use systems::fsm_transition_system;

pub struct FsmPlugin;

impl Plugin for FsmPlugin {
    fn build(&self, app: &mut App) {
        let mob_registry = load_mobs("assets/mobs");

        app.insert_resource(mob_registry)
            .add_message::<StateTransition>()
            .add_systems(
                Update,
                (
                    move_toward_player_system,
                    use_abilities_system,
                    when_near_system,
                    after_time_system,
                    fsm_transition_system,
                )
                    .chain(),
            );
    }
}
