use bevy::{app::AppExit, prelude::*};

use crate::game_state::GameState;

#[derive(Component)]
pub(super) enum MenuButton {
    Play,
    Exit,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub(super) fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::MainMenu),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.9)),
                children![
                    (
                        Text::new("Magic Craft"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(50.0)),
                            ..default()
                        }
                    ),
                    (
                        Button,
                        MenuButton::Play,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Play"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR)
                        )]
                    ),
                    (
                        Button,
                        MenuButton::Exit,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Exit"),
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

pub(super) fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match button {
                    MenuButton::Play => next_state.set(GameState::Playing),
                    MenuButton::Exit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
