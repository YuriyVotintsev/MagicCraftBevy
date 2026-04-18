mod button;
mod panel;

use bevy::prelude::*;

pub use button::{button_node, ReleasedButtons};
pub use panel::panel_node;

pub fn register(app: &mut App) {
    app.add_systems(Update, button::button_interaction_visuals)
        .add_systems(PostUpdate, button::update_button_last_interaction);
}
