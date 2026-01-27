mod damage_numbers;
mod game_over;
mod hud;
mod loading;
mod main_menu;
mod shop;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::wave::WavePhase;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), loading::spawn_loading_screen)
            .add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                main_menu::menu_button_system.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over::spawn_game_over)
            .add_systems(
                Update,
                game_over::game_over_button_system.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnEnter(GameState::Playing), hud::spawn_hud)
            .add_systems(
                Update,
                hud::update_hud.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(WavePhase::Shop), shop::spawn_shop)
            .add_systems(
                Update,
                shop::shop_button_system.run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                Update,
                (
                    damage_numbers::spawn_damage_numbers,
                    damage_numbers::update_damage_numbers,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
