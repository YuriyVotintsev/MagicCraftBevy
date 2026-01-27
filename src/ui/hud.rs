use bevy::prelude::*;

use crate::money::PlayerMoney;
use crate::player::Player;
use crate::stats::{ComputedStats, Health, StatRegistry};
use crate::wave::WaveState;
use crate::GameState;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct KillCountText;

#[derive(Component)]
pub struct KillProgressBar;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct HealthBar;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BAR_BG_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const HEALTH_BAR_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
const KILL_BAR_COLOR: Color = Color::srgb(0.2, 0.6, 0.8);

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
                WaveText,
                Text::new("Wave: 1"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
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
                TextColor(Color::srgb(1.0, 0.84, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                }
            ),
            (
                KillCountText,
                Text::new("Enemies: 0/5"),
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
                    height: Val::Px(12.0),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(BAR_BG_COLOR),
                children![(
                    KillProgressBar,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(KILL_BAR_COLOR)
                )]
            ),
            (
                HealthText,
                Text::new("HP: 100/100"),
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
                    HealthBar,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(HEALTH_BAR_COLOR)
                )]
            ),
        ],
    ));
}

pub fn update_hud(
    wave_state: Res<WaveState>,
    money: Res<PlayerMoney>,
    stat_registry: Res<StatRegistry>,
    player_query: Query<(&Health, &ComputedStats), With<Player>>,
    mut wave_text: Query<&mut Text, (With<WaveText>, Without<MoneyText>, Without<KillCountText>, Without<HealthText>)>,
    mut money_text: Query<&mut Text, (With<MoneyText>, Without<WaveText>, Without<KillCountText>, Without<HealthText>)>,
    mut kill_text: Query<&mut Text, (With<KillCountText>, Without<WaveText>, Without<MoneyText>, Without<HealthText>)>,
    mut kill_bar: Query<&mut Node, (With<KillProgressBar>, Without<HealthBar>)>,
    mut health_text: Query<&mut Text, (With<HealthText>, Without<WaveText>, Without<MoneyText>, Without<KillCountText>)>,
    mut health_bar: Query<&mut Node, (With<HealthBar>, Without<KillProgressBar>)>,
) {
    if let Ok(mut text) = wave_text.single_mut() {
        **text = format!("Wave: {}", wave_state.current_wave);
    }

    if let Ok(mut text) = money_text.single_mut() {
        **text = format!("Coins: {}", money.0);
    }

    if let Ok(mut text) = kill_text.single_mut() {
        **text = format!(
            "Enemies: {}/{}",
            wave_state.killed_count, wave_state.target_count
        );
    }

    if let Ok(mut node) = kill_bar.single_mut() {
        let progress = if wave_state.target_count > 0 {
            (wave_state.killed_count as f32 / wave_state.target_count as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
        node.width = Val::Percent(progress);
    }

    if let Ok((health, stats)) = player_query.single() {
        let max_life = stat_registry
            .get("max_life")
            .map(|id| stats.get(id))
            .unwrap_or(100.0);

        if let Ok(mut text) = health_text.single_mut() {
            **text = format!("HP: {}/{}", health.current as i32, max_life as i32);
        }

        if let Ok(mut node) = health_bar.single_mut() {
            let progress = (health.current / max_life * 100.0).clamp(0.0, 100.0);
            node.width = Val::Percent(progress);
        }
    }
}
