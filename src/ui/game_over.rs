use bevy::prelude::*;

use crate::game_state::GameState;

#[derive(Component)]
pub(super) enum GameOverButton {
    Retry,
    MainMenu,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub(super) fn spawn_game_over(mut commands: Commands) {
    commands.spawn((
        Name::new("GameOverRoot"),
        DespawnOnExit(GameState::GameOver),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.1, 0.1, 0.9)),
                children![
                    (
                        Text::new("Game Over"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.3, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(50.0)),
                            ..default()
                        }
                    ),
                    (
                        Button,
                        GameOverButton::Retry,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Retry"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR)
                        )]
                    ),
                    (
                        Button,
                        GameOverButton::MainMenu,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Main Menu"),
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

pub(super) fn game_over_button_system(
    mut interaction_query: Query<
        (&Interaction, &GameOverButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match button {
                    GameOverButton::Retry => next_state.set(GameState::HeroSelection),
                    GameOverButton::MainMenu => next_state.set(GameState::MainMenu),
                }
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
