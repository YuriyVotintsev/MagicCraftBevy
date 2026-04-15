use bevy::prelude::*;

use crate::game_state::GameState;

const BG_COLOR: Color = Color::srgba(0.04, 0.02, 0.05, 0.95);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const TITLE_COLOR: Color = Color::srgb(0.95, 0.35, 0.35);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub enum GameOverButton {
    NewRun,
    MainMenu,
}

pub fn spawn_game_over_screen(mut commands: Commands) {
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
        BackgroundColor(BG_COLOR),
        children![
            (
                Text::new("Game Over"),
                TextFont { font_size: 72.0, ..default() },
                TextColor(TITLE_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            (
                Button,
                GameOverButton::NewRun,
                button_node(),
                BackgroundColor(NORMAL_BUTTON),
                children![(
                    Text::new("New Run"),
                    TextFont { font_size: 32.0, ..default() },
                    TextColor(TEXT_COLOR),
                )],
            ),
            (
                Button,
                GameOverButton::MainMenu,
                button_node(),
                BackgroundColor(NORMAL_BUTTON),
                children![(
                    Text::new("Main Menu"),
                    TextFont { font_size: 32.0, ..default() },
                    TextColor(TEXT_COLOR),
                )],
            ),
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

pub fn game_over_button_system(
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
                    GameOverButton::NewRun => next_state.set(GameState::Playing),
                    GameOverButton::MainMenu => next_state.set(GameState::MainMenu),
                }
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
