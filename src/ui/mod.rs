mod dev_menu;
mod game_over;
mod hud;
mod loading;
mod main_menu;
mod pause_menu;
mod shop_view;
mod stat_line_builder;

use bevy::prelude::*;
use bevy::ui::{UiScale, UiSystems};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::arena::WINDOW_HEIGHT;
use crate::game_state::GameState;
use crate::rune::fill_shop_offer;
use crate::wave::{CombatPhase, WavePhase};

pub const PANEL_RADIUS: f32 = 20.0;
pub fn panel_radius() -> BorderRadius {
    BorderRadius::all(Val::Px(PANEL_RADIUS))
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Viewport>()
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
                hud::update_hud
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                OnEnter(WavePhase::Shop),
                (fill_shop_offer, shop_view::spawn_shop_screen).chain(),
            )
            .add_systems(
                OnExit(WavePhase::Shop),
                shop_view::restore_dragged_on_exit,
            )
            .add_systems(
                Update,
                (
                    shop_view::start_run_system,
                    shop_view::update_coins_text,
                    shop_view::update_shop_price_labels,
                    shop_view::reposition_shop_ui,
                    (
                        shop_view::start_drag,
                        shop_view::finish_drag,
                        shop_view::reconcile_rune_entities,
                        shop_view::sync_cell_lock_visuals,
                        shop_view::update_highlights,
                        shop_view::apply_highlights,
                    )
                        .chain(),
                )
                    .run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                PostUpdate,
                shop_view::follow_cursor
                    .before(UiSystems::Layout)
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
                    dev_menu::cheat_win_wave,
                    dev_menu::toggle_enemy_type,
                    dev_menu::enable_all_enemies,
                    dev_menu::disable_all_enemies,
                )
                    .run_if(in_state(CombatPhase::DevMenu)),
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
