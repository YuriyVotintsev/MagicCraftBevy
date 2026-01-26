use bevy::prelude::*;

use crate::money::PlayerMoney;
use crate::wave::{WavePhase, WaveState};

#[derive(Component)]
pub struct ShopRoot;

#[derive(Component)]
pub struct ShopButton;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub fn spawn_shop(
    mut commands: Commands,
    wave_state: Res<WaveState>,
    money: Res<PlayerMoney>,
) {
    commands.spawn((
        ShopRoot,
        DespawnOnExit(WavePhase::Shop),
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
                BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.95)),
                children![
                    (
                        Text(format!("Wave {} Complete!", wave_state.current_wave)),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        }
                    ),
                    (
                        Text(format!("Total: {} coins", money.0)),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.84, 0.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(40.0)),
                            ..default()
                        }
                    ),
                    (
                        Button,
                        ShopButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON),
                        children![(
                            Text::new("Next Wave"),
                            TextFont {
                                font_size: 28.0,
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

pub fn shop_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ShopButton>),
    >,
    mut next_phase: ResMut<NextState<WavePhase>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_phase.set(WavePhase::Combat);
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
