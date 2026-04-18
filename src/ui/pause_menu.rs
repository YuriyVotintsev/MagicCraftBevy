use bevy::prelude::*;

use crate::game_state::GameState;
use crate::palette;
use crate::wave::CombatPhase;

use super::widgets::{button_node, panel_node, ReleasedButtons};

#[derive(Component)]
pub(super) enum PauseButton {
    Continue,
    EndRun,
}

pub(super) fn spawn_pause_menu(mut commands: Commands) {
    let text = palette::color("ui_text");
    commands.spawn((
        Name::new("PauseMenuRoot"),
        DespawnOnExit(CombatPhase::Paused),
        GlobalZIndex(100),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(palette::color_alpha("ui_overlay_bg", 0.6)),
        children![
            (
                panel_node(
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        ..default()
                    },
                    None,
                ),
                children![
                    (
                        Text::new("Paused"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        TextColor(palette::color("ui_text_title")),
                        Node {
                            margin: UiRect::bottom(Val::Px(50.0)),
                            ..default()
                        }
                    ),
                    (
                        PauseButton::Continue,
                        button_node(menu_button_node(), None),
                        children![(
                            Text::new("Continue"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(text)
                        )]
                    ),
                    (
                        PauseButton::EndRun,
                        button_node(menu_button_node(), None),
                        children![(
                            Text::new("End Run"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(text)
                        )]
                    ),
                ]
            )
        ],
    ));
}

fn menu_button_node() -> Node {
    Node {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub(super) fn toggle_pause_system(
    key: Res<ButtonInput<KeyCode>>,
    combat_phase: Res<State<CombatPhase>>,
    mut next_phase: ResMut<NextState<CombatPhase>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if key.just_pressed(KeyCode::Escape) {
        match combat_phase.get() {
            CombatPhase::Running => {
                virtual_time.pause();
                next_phase.set(CombatPhase::Paused);
            }
            CombatPhase::Paused => {
                virtual_time.unpause();
                next_phase.set(CombatPhase::Running);
            }
            CombatPhase::DevMenu => {}
        }
    }
}

pub(super) fn pause_button_system(
    buttons: ReleasedButtons<PauseButton>,
    mut next_phase: ResMut<NextState<CombatPhase>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    buttons.for_each(|button| match button {
        PauseButton::Continue => {
            virtual_time.unpause();
            next_phase.set(CombatPhase::Running);
        }
        PauseButton::EndRun => {
            virtual_time.unpause();
            next_game_state.set(GameState::MainMenu);
        }
    });
}
