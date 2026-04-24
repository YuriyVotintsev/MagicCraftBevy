use bevy::prelude::*;

mod keyboard_movement;
mod player_input;

pub use keyboard_movement::{KeyboardMovement, MovementLocked};
pub use player_input::{PlayerAbilityCooldowns, PlayerInput};

pub struct PlayerComponentsPlugin;

impl Plugin for PlayerComponentsPlugin {
    fn build(&self, app: &mut App) {
        keyboard_movement::register_systems(app);
        player_input::register_systems(app);
    }
}
