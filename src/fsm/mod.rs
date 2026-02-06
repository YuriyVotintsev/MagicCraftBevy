mod events;

pub use events::StateTransition;

use bevy::prelude::*;

pub struct FsmPlugin;

impl Plugin for FsmPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StateTransition>();
    }
}
