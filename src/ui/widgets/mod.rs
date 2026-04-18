mod button;
mod panel;

use bevy::prelude::*;

pub use button::button_node;
pub use panel::panel_node;

pub fn register(app: &mut App) {
    app.add_systems(Update, button::button_interaction_visuals);
}
