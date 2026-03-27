use bevy::prelude::*;

use crate::money::PlayerMoney;
use crate::player::Player;
use crate::blueprints::components::common::health::Health;
use crate::stats::{ComputedStats, StatRegistry};
use crate::GameState;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct StaminaText;

#[derive(Component)]
pub struct StaminaBar;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BAR_BG_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const STAMINA_BAR_COLOR: Color = Color::srgb(0.3, 0.8, 0.3);

pub fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("HudRoot"),
        HudRoot,
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        children![
            (
                MoneyText,
                Text::new("Coins: 0"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.84, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                }
            ),
            (
                StaminaText,
                Text::new("Stamina: 0/0"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(3.0)),
                    ..default()
                }
            ),
            (
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(BAR_BG_COLOR),
                children![(
                    StaminaBar,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(STAMINA_BAR_COLOR)
                )]
            ),
        ],
    ));
}

pub fn update_hud(
    money: Res<PlayerMoney>,
    stat_registry: Res<StatRegistry>,
    player_query: Query<(&Health, &ComputedStats), With<Player>>,
    mut money_text: Query<&mut Text, (With<MoneyText>, Without<StaminaText>)>,
    mut stamina_text: Query<&mut Text, (With<StaminaText>, Without<MoneyText>)>,
    mut stamina_bar: Query<&mut Node, With<StaminaBar>>,
) {
    if let Ok(mut text) = money_text.single_mut() {
        **text = format!("Coins: {}", money.get());
    }

    if let Ok((health, stats)) = player_query.single() {
        let max_stamina = stat_registry
            .get("max_stamina")
            .map(|id| stats.get(id))
            .unwrap_or_default();

        if let Ok(mut text) = stamina_text.single_mut() {
            **text = format!("Stamina: {}/{}", health.current as i32, max_stamina as i32);
        }

        if let Ok(mut node) = stamina_bar.single_mut() {
            let progress = if max_stamina > 0.0 {
                (health.current / max_stamina * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            };
            node.width = Val::Percent(progress);
        }
    }
}
