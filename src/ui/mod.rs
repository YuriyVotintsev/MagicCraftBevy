mod game_over;
mod main_menu;

use bevy::prelude::*;

use crate::game_state::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                main_menu::menu_button_system.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over::spawn_game_over)
            .add_systems(
                Update,
                game_over::game_over_button_system.run_if(in_state(GameState::GameOver)),
            );
    }
}
