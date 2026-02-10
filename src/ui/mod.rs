mod artifact_panel;
mod artifact_tooltip;
mod damage_numbers;
mod game_over;
mod hero_selection;
mod hud;
mod loading;
mod main_menu;
mod shop;
mod spell_selection;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::schedule::GameSet;
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
            .add_systems(
                OnEnter(GameState::HeroSelection),
                hero_selection::spawn_hero_selection,
            )
            .add_systems(
                Update,
                (
                    hero_selection::hero_button_system,
                    hero_selection::update_hero_button_colors,
                    hero_selection::update_stats_panel,
                    hero_selection::continue_button_system,
                )
                    .run_if(in_state(GameState::HeroSelection)),
            )
            .add_systems(
                OnEnter(GameState::SpellSelection),
                spell_selection::spawn_spell_selection,
            )
            .add_systems(
                Update,
                (
                    spell_selection::spell_button_system,
                    spell_selection::update_spell_button_colors,
                    spell_selection::start_button_system,
                )
                    .run_if(in_state(GameState::SpellSelection)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over::spawn_game_over)
            .add_systems(
                Update,
                game_over::game_over_button_system.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                (hud::spawn_hud, artifact_panel::spawn_artifact_panel),
            )
            .add_systems(
                Update,
                (
                    hud::update_hud.after(GameSet::Damage),
                    artifact_panel::rebuild_artifact_panel,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(WavePhase::Shop), shop::spawn_shop)
            .add_systems(
                Update,
                (
                    (
                        shop::buy_system,
                        artifact_panel::handle_artifact_slot_click,
                        artifact_panel::handle_panel_sell_click,
                    ),
                    shop::update_shop_on_change,
                )
                    .chain()
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                Update,
                (
                    shop::next_wave_system,
                    shop::update_button_colors,
                    artifact_panel::update_panel_button_colors,
                )
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                OnExit(WavePhase::Shop),
                artifact_panel::clear_artifact_selection,
            )
            .add_systems(
                Update,
                (
                    damage_numbers::spawn_damage_numbers,
                    damage_numbers::update_damage_numbers,
                    artifact_tooltip::update_artifact_tooltip,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
