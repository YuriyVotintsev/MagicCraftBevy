use bevy::{app::AppExit, prelude::*};

use crate::game_state::GameState;
use crate::palette;
use crate::transition::{Transition, TransitionAction};

use super::panel_radius;

#[derive(Component)]
pub(super) enum MenuButton {
    Play,
    Exit,
}

pub(super) fn spawn_main_menu(mut commands: Commands) {
    let text = palette::color("ui_text");
    let panel = palette::color("ui_panel_bg");
    let button = palette::color("ui_button_normal");
    commands.spawn((
        Name::new("MainMenuRoot"),
        DespawnOnExit(GameState::MainMenu),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(palette::color("ui_screen_bg")),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    border_radius: panel_radius(),
                    ..default()
                },
                BackgroundColor(panel),
                children![
                    (
                        Text::new("Magic Craft"),
                        TextFont {
                            font_size: 64.0,
                            ..default()
                        },
                        TextColor(text),
                        Node {
                            margin: UiRect::bottom(Val::Px(50.0)),
                            ..default()
                        }
                    ),
                    (
                        Button,
                        MenuButton::Play,
                        button_node(),
                        BackgroundColor(button),
                        children![(
                            Text::new("Play"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(text)
                        )]
                    ),
                    (
                        Button,
                        MenuButton::Exit,
                        button_node(),
                        BackgroundColor(button),
                        children![(
                            Text::new("Exit"),
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

fn button_node() -> Node {
    Node {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: panel_radius(),
        ..default()
    }
}

pub(super) fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut transition: ResMut<Transition>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = palette::color("ui_button_pressed").into();
                match button {
                    MenuButton::Play => {
                        transition.request(TransitionAction::Game(GameState::Playing));
                    }
                    MenuButton::Exit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => *color = palette::color("ui_button_hover").into(),
            Interaction::None => *color = palette::color("ui_button_normal").into(),
        }
    }
}
