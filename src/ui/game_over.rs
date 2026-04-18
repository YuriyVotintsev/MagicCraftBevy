use bevy::prelude::*;

use crate::game_state::GameState;
use crate::palette;
use crate::transition::{Transition, TransitionAction};

use super::widgets::button_node;

#[derive(Component)]
pub enum GameOverButton {
    NewRun,
    MainMenu,
}

pub fn spawn_game_over_screen(mut commands: Commands) {
    let text = palette::color("ui_text");
    commands.spawn((
        Name::new("GameOverRoot"),
        DespawnOnExit(GameState::GameOver),
        GlobalZIndex(100),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(palette::color("ui_screen_bg")),
        children![
            (
                Text::new("Game Over"),
                TextFont { font_size: 72.0, ..default() },
                TextColor(palette::color("ui_text_gameover")),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            (
                GameOverButton::NewRun,
                button_node(menu_button_node(), None),
                children![(
                    Text::new("New Run"),
                    TextFont { font_size: 32.0, ..default() },
                    TextColor(text),
                )],
            ),
            (
                GameOverButton::MainMenu,
                button_node(menu_button_node(), None),
                children![(
                    Text::new("Main Menu"),
                    TextFont { font_size: 32.0, ..default() },
                    TextColor(text),
                )],
            ),
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

pub fn game_over_button_system(
    interaction_query: Query<(&Interaction, &GameOverButton), Changed<Interaction>>,
    mut transition: ResMut<Transition>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction != Interaction::Pressed { continue }
        let target = match button {
            GameOverButton::NewRun => GameState::Playing,
            GameOverButton::MainMenu => GameState::MainMenu,
        };
        transition.request(TransitionAction::Game(target));
    }
}
