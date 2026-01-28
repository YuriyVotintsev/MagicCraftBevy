mod interval;
mod on_input;
mod passive;
mod while_held;

pub use interval::{IntervalActivations, interval_system};
pub use on_input::{OnInputActivations, on_input_system};
pub use passive::{PassiveActivations, passive_system};
pub use while_held::{WhileHeldActivations, while_held_system};

use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::GameState;

pub fn register_activator_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            on_input_system,
            passive_system,
            while_held_system,
            interval_system,
        )
            .in_set(GameSet::AbilityActivation)
            .run_if(in_state(GameState::Playing)),
    );
}
