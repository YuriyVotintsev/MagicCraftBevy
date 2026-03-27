mod affix_shop;
mod artifact_panel;
mod artifact_tooltip;
mod damage_numbers;
mod game_over;
mod hero_selection;
mod hud;
mod loading;
mod main_menu;
mod pause_menu;
mod shop;
pub mod skill_tree_view;
mod spell_selection;
pub mod stat_line_builder;

use bevy::prelude::*;

use crate::game_state::GameState;
use crate::schedule::ShopSet;
use crate::wave::{CombatPhase, WavePhase};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<skill_tree_view::PanState>()
            .init_resource::<skill_tree_view::ZoomLevel>()
            .add_systems(OnEnter(GameState::Loading), loading::spawn_loading_screen)
            .add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                main_menu::menu_button_system.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(Startup, skill_tree_view::setup_skill_tree_meshes)
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
                skill_tree_view::spawn_shop_screen,
            )
            .add_systems(
                Update,
                skill_tree_view::skill_tree_click
                    .in_set(ShopSet::Input)
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                Update,
                (
                    skill_tree_view::start_run_system,
                    skill_tree_view::update_node_visuals,
                    skill_tree_view::skill_tree_pan_zoom,
                    skill_tree_view::skill_tree_hover,
                    skill_tree_view::update_skill_points_text,
                    skill_tree_view::update_coins_text,
                )
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                OnExit(WavePhase::Shop),
                skill_tree_view::cleanup_skill_tree_view,
            )
            .add_systems(
                Update,
                (
                    damage_numbers::spawn_damage_numbers,
                    damage_numbers::update_damage_numbers,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(CombatPhase::Paused), pause_menu::spawn_pause_menu)
            .add_systems(
                Update,
                pause_menu::toggle_pause_system.run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                Update,
                pause_menu::pause_button_system.run_if(in_state(CombatPhase::Paused)),
            );
    }
}
