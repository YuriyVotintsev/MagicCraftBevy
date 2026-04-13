use bevy::prelude::*;

use crate::run::money::PlayerMoney;
use crate::run::RunState;
use crate::wave::WavePhase;

const BG_COLOR: Color = Color::srgb(0.03, 0.03, 0.08);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct StartRunButton;

pub fn spawn_shop_screen(
    mut commands: Commands,
    run_state: Res<RunState>,
    money: Res<PlayerMoney>,
) {
    commands.spawn((
        DespawnOnExit(WavePhase::Shop),
        GlobalZIndex(50),
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
                Text(format!("Run {}", run_state.attempt)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ),
            (
                ShopCoinsText,
                Text(format!("Coins: {}", money.get())),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(GOLD_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ),
            (
                Button,
                StartRunButton,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                children![(
                    Text::new("Start Run"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                )]
            ),
        ],
    ));
}

pub fn start_run_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartRunButton>),
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

pub fn update_coins_text(
    money: Res<PlayerMoney>,
    mut text_query: Query<&mut Text, With<ShopCoinsText>>,
) {
    if !money.is_changed() {
        return;
    }
    for mut text in &mut text_query {
        text.0 = format!("Coins: {}", money.get());
    }
}
