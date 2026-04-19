mod dev_menu;
mod game_over;
mod hud;
mod loading;
mod main_menu;
mod pause_menu;
mod shop_hud;
mod stat_line_builder;
mod stats_panel;
mod widgets;

use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::{PrimaryWindow, WindowResized};

use crate::arena::WINDOW_HEIGHT;
use crate::game_state::GameState;
use crate::wave::{CombatPhase, WavePhase};

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        widgets::register(app);
        shop_hud::register(app);
        app.init_resource::<Viewport>()
            .init_resource::<stats_panel::StatsPanelState>()
            .add_systems(Startup, init_layout)
            .add_systems(Update, update_layout_on_resize)
            .add_systems(OnEnter(GameState::Loading), loading::spawn_loading_screen)
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
                hud::update_hud.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnEnter(WavePhase::Shop),
                stats_panel::spawn_stats_panel,
            )
            .add_systems(
                Update,
                (stats_panel::compute_state, stats_panel::render)
                    .chain()
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(OnEnter(GameState::GameOver), game_over::spawn_game_over_screen)
            .add_systems(
                Update,
                game_over::game_over_button_system.run_if(in_state(GameState::GameOver)),
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
            .init_resource::<dev_menu::ShopDevMenuOpen>()
            .add_systems(OnEnter(CombatPhase::DevMenu), dev_menu::spawn_dev_menu)
            .add_systems(OnExit(WavePhase::Shop), dev_menu::reset_shop_dev_menu)
            .add_systems(
                Update,
                dev_menu::toggle_dev_menu.run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                Update,
                (dev_menu::toggle_shop_dev_menu, dev_menu::react_to_shop_dev_menu)
                    .chain()
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                Update,
                (
                    dev_menu::camera_angle_slider_interaction,
                    dev_menu::camera_zoom_slider_interaction,
                    dev_menu::cheat_money,
                    dev_menu::cheat_health,
                    dev_menu::cheat_damage,
                    dev_menu::cheat_win_wave,
                    dev_menu::toggle_enemy_type,
                    dev_menu::enable_all_enemies,
                    dev_menu::disable_all_enemies,
                )
                    .run_if(dev_menu::dev_menu_active),
            );
    }
}

fn init_layout(
    mut ui_scale: ResMut<UiScale>,
    mut viewport: ResMut<Viewport>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    apply_layout(window.width(), window.height(), &mut ui_scale, &mut viewport);
}

fn update_layout_on_resize(
    mut events: MessageReader<WindowResized>,
    mut ui_scale: ResMut<UiScale>,
    mut viewport: ResMut<Viewport>,
) {
    if let Some(last) = events.read().last() {
        apply_layout(last.width, last.height, &mut ui_scale, &mut viewport);
    }
}

fn apply_layout(
    win_w: f32,
    win_h: f32,
    ui_scale: &mut UiScale,
    viewport: &mut Viewport,
) {
    let scale = if win_h > 0.0 { win_h / WINDOW_HEIGHT } else { 1.0 };
    ui_scale.0 = scale;
    viewport.height = WINDOW_HEIGHT;
    viewport.width = if scale > 0.0 { win_w / scale } else { win_w };
}
