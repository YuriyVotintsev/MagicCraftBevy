pub mod hero_class;
pub mod selected_spells;
mod systems;

use bevy::prelude::*;

use crate::wave::WavePhase;

pub use hero_class::{AvailableHeroes, SelectedHero};
pub use selected_spells::{SelectedSpells, SpellSlot};
pub use systems::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSpells>()
            .init_resource::<SelectedHero>()
            .add_systems(OnEnter(WavePhase::Combat), systems::spawn_player)
            .add_systems(OnExit(WavePhase::Combat), systems::reset_player_velocity);
    }
}
