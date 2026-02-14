pub mod hero_class;
pub mod selected_spells;
mod systems;

use bevy::prelude::*;

use crate::GameState;
use crate::schedule::PostGameSet;
use crate::stats::death_system;
use crate::wave::WavePhase;

pub use hero_class::{AvailableHeroes, SelectedHero};
pub use selected_spells::{SelectedSpells, SpellSlot};
pub use systems::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSpells>()
            .init_resource::<SelectedHero>()
            .add_systems(OnEnter(GameState::Playing), systems::spawn_player)
            .add_systems(OnExit(WavePhase::Combat), systems::reset_player_velocity)
            .add_systems(PostUpdate, systems::handle_player_death.after(death_system).in_set(PostGameSet));
    }
}
