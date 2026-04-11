mod dev_menu;
mod hud;
mod loading;
mod main_menu;
mod pause_menu;
mod shop_view;
pub mod stat_line_builder;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::wave::{CombatPhase, WavePhase};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), loading::spawn_loading_screen)
            .add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                main_menu::menu_button_system.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                hud::spawn_hud,
            )
            .add_systems(
                Update,
                hud::update_hud
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                OnEnter(WavePhase::Shop),
                shop_view::spawn_shop_screen,
            )
            .add_systems(
                Update,
                (
                    shop_view::start_run_system,
                    shop_view::update_coins_text,
                )
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(OnEnter(CombatPhase::Paused), pause_menu::spawn_pause_menu)
            .add_systems(
                Update,
                pause_menu::toggle_pause_system.run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                Update,
                pause_menu::pause_button_system.run_if(in_state(CombatPhase::Paused)),
            )
            .add_systems(OnEnter(CombatPhase::DevMenu), dev_menu::spawn_dev_menu)
            .add_systems(
                Update,
                dev_menu::toggle_dev_menu.run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                Update,
                (
                    dev_menu::slider_interaction,
                    dev_menu::cheat_money,
                    dev_menu::cheat_health,
                    dev_menu::cheat_damage,
                    dev_menu::toggle_enemy_type,
                    dev_menu::enable_all_enemies,
                    dev_menu::disable_all_enemies,
                )
                    .run_if(in_state(CombatPhase::DevMenu)),
            );
    }
}
