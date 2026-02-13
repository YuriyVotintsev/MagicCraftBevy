use bevy::prelude::*;

use crate::game_state::GameState;
use crate::wave::CombatPhase;

#[derive(Component)]
pub(super) enum PauseButton {
    Continue,
    EndRun,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub(super) fn spawn_pause_menu(mut commands: Commands) {
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                children![
                    (
                        Text::new("Paused"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.9)),
                        Node {
                            margin: UiRect::bottom(Val::Px(50.0)),
                            ..default()
                        }
                    ),
                    (
                        Button,
                        PauseButton::Continue,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Continue"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR)
                        )]
                    ),
                    (
                        Button,
                        PauseButton::EndRun,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("End Run"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR)
                        )]
                    ),
                ]
            )
        ],
    ));
}

fn button_node() -> Node {
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
        }
    }
}

pub(super) fn pause_button_system(
    mut interaction_query: Query<
        (&Interaction, &PauseButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_phase: ResMut<NextState<CombatPhase>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match button {
                    PauseButton::Continue => {
                        virtual_time.unpause();
                        next_phase.set(CombatPhase::Running);
                    }
                    PauseButton::EndRun => {
                        virtual_time.unpause();
                        next_game_state.set(GameState::GameOver);
                    }
                }
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
