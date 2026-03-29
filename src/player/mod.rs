pub mod hero_class;
pub mod selected_spells;
mod sphere_visual;
mod systems;

use bevy::prelude::*;

use crate::wave::WavePhase;

pub use hero_class::AvailableHeroes;
pub use selected_spells::{SelectedSpells, SpellSlot};
pub use systems::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSpells>()
            .add_systems(OnEnter(WavePhase::Combat), systems::spawn_player)
            .add_systems(OnExit(WavePhase::Combat), systems::reset_player_velocity);
        sphere_visual::register_systems(app);
    }
}
