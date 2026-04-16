use bevy::prelude::*;

use crate::game_state::GameState;
use crate::palette;

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
        BackgroundColor(palette::color("ui_screen_bg")),
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
                        TextColor(palette::color("ui_text")),
                    ),
                ]
            )
        ],
    ));
}
