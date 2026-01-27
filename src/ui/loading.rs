use bevy::prelude::*;

use crate::game_state::GameState;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub(super) fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        Name::new("LoadingRoot"),
        DespawnOnExit(GameState::Loading),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Text::new("Loading..."),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ),
                ]
            )
        ],
    ));
}
