use bevy::prelude::*;

use crate::actors::Health;
use crate::actors::Player;
use crate::palette;
use crate::run::{PlayerMoney, RunState};
use crate::stats::{ComputedStats, Stat};
use crate::GameState;

use super::widgets::panel_node;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct LifeText;

#[derive(Component)]
pub struct LifeBar;

pub fn spawn_hud(mut commands: Commands, run_state: Res<RunState>) {
    commands.spawn((
        Name::new("HudRoot"),
        HudRoot,
        DespawnOnExit(GameState::Playing),
        panel_node(
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            None,
        ),
        children![
            (
                WaveText,
                Text::new(format!("Wave {}", run_state.wave)),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(palette::color("ui_text")),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                }
            ),
            (
                MoneyText,
                Text::new("Coins: 0"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(palette::color("ui_text_money")),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                }
            ),
            (
                LifeText,
                Text::new("Life: 0/0"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(palette::color("ui_text")),
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
                BackgroundColor(palette::color("ui_lifebar_bg")),
                children![(
                    LifeBar,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(palette::color("ui_lifebar"))
                )]
            ),
        ],
    ));
}

pub fn update_hud(
    money: Res<PlayerMoney>,
    run_state: Res<RunState>,
    player_query: Query<(&Health, &ComputedStats), With<Player>>,
    mut wave_text: Query<&mut Text, (With<WaveText>, Without<MoneyText>, Without<LifeText>)>,
    mut money_text: Query<&mut Text, (With<MoneyText>, Without<LifeText>, Without<WaveText>)>,
    mut life_text: Query<&mut Text, (With<LifeText>, Without<MoneyText>, Without<WaveText>)>,
    mut life_bar: Query<&mut Node, With<LifeBar>>,
) {
    if let Ok(mut text) = wave_text.single_mut() {
        **text = format!("Wave {}", run_state.wave);
    }

    if let Ok(mut text) = money_text.single_mut() {
        **text = format!("Coins: {}", money.get());
    }

    if let Ok((health, stats)) = player_query.single() {
        let max_life = stats.final_of(Stat::MaxLife);

        if let Ok(mut text) = life_text.single_mut() {
            **text = format!("Life: {}/{}", health.current as i32, max_life as i32);
        }

        if let Ok(mut node) = life_bar.single_mut() {
            let progress = if max_life > 0.0 {
                (health.current / max_life * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            };
            node.width = Val::Percent(progress);
        }
    }
}
