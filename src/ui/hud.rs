use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::arena::RenderSettings;
use crate::money::PlayerMoney;
use crate::player::Player;
use crate::blueprints::components::common::health::Health;
use crate::stats::{ComputedStats, StatRegistry};
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

#[derive(Component)]
pub struct CameraAngleSlider;

#[derive(Component)]
pub struct CameraAngleFill;

#[derive(Component)]
pub struct CameraAngleLabel;

#[derive(Component)]
pub struct SpriteTiltSlider;

#[derive(Component)]
pub struct SpriteTiltFill;

#[derive(Component)]
pub struct SpriteTiltLabel;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const BAR_BG_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const HEALTH_BAR_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
const KILL_BAR_COLOR: Color = Color::srgb(0.2, 0.6, 0.8);
const SLIDER_COLOR: Color = Color::srgb(0.4, 0.6, 0.9);

pub fn spawn_hud(mut commands: Commands, settings: Res<RenderSettings>) {
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

    let camera_t = (settings.camera_angle - 10.0) / 80.0;
    let tilt_t = settings.sprite_tilt / 90.0;

    commands.spawn((
        Name::new("SliderPanel"),
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(6.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        children![
            (
                CameraAngleLabel,
                Text::new(format!("Camera: {:.0}", settings.camera_angle)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(TEXT_COLOR),
            ),
            (
                CameraAngleSlider,
                Interaction::default(),
                RelativeCursorPosition::default(),
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(BAR_BG_COLOR),
                children![(
                    CameraAngleFill,
                    Pickable::IGNORE,
                    Node {
                        width: Val::Percent(camera_t * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SLIDER_COLOR),
                )]
            ),
            (
                SpriteTiltLabel,
                Text::new(format!("Tilt: {:.0}", settings.sprite_tilt)),
                TextFont { font_size: 16.0, ..default() },
                TextColor(TEXT_COLOR),
            ),
            (
                SpriteTiltSlider,
                Interaction::default(),
                RelativeCursorPosition::default(),
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(BAR_BG_COLOR),
                children![(
                    SpriteTiltFill,
                    Pickable::IGNORE,
                    Node {
                        width: Val::Percent(tilt_t * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SLIDER_COLOR),
                )]
            ),
        ],
    ));
}

pub fn slider_system(
    camera_track: Query<(&Interaction, &RelativeCursorPosition), With<CameraAngleSlider>>,
    tilt_track: Query<(&Interaction, &RelativeCursorPosition), With<SpriteTiltSlider>>,
    mut settings: ResMut<RenderSettings>,
    mut camera_label: Query<&mut Text, (With<CameraAngleLabel>, Without<SpriteTiltLabel>)>,
    mut camera_fill: Query<&mut Node, (With<CameraAngleFill>, Without<SpriteTiltFill>, Without<KillProgressBar>, Without<HealthBar>)>,
    mut tilt_label: Query<&mut Text, (With<SpriteTiltLabel>, Without<CameraAngleLabel>)>,
    mut tilt_fill: Query<&mut Node, (With<SpriteTiltFill>, Without<CameraAngleFill>, Without<KillProgressBar>, Without<HealthBar>)>,
) {
    if let Ok((interaction, cursor)) = camera_track.single() {
        if *interaction == Interaction::Pressed {
            if let Some(pos) = cursor.normalized {
                let t = (pos.x + 0.5).clamp(0.0, 1.0);
                settings.camera_angle = 10.0 + t * 80.0;
            }
        }
    }

    if let Ok((interaction, cursor)) = tilt_track.single() {
        if *interaction == Interaction::Pressed {
            if let Some(pos) = cursor.normalized {
                let t = (pos.x + 0.5).clamp(0.0, 1.0);
                settings.sprite_tilt = t * 90.0;
            }
        }
    }

    if settings.is_changed() {
        if let Ok(mut text) = camera_label.single_mut() {
            **text = format!("Camera: {:.0}", settings.camera_angle);
        }
        if let Ok(mut node) = camera_fill.single_mut() {
            let t = (settings.camera_angle - 10.0) / 80.0;
            node.width = Val::Percent(t * 100.0);
        }
        if let Ok(mut text) = tilt_label.single_mut() {
            **text = format!("Tilt: {:.0}", settings.sprite_tilt);
        }
        if let Ok(mut node) = tilt_fill.single_mut() {
            let t = settings.sprite_tilt / 90.0;
            node.width = Val::Percent(t * 100.0);
        }
    }
}

pub fn update_hud(
    wave_state: Res<WaveState>,
    money: Res<PlayerMoney>,
    stat_registry: Res<StatRegistry>,
    player_query: Query<(&Health, &ComputedStats), With<Player>>,
    mut wave_text: Query<&mut Text, (With<WaveText>, Without<MoneyText>, Without<KillCountText>, Without<HealthText>)>,
    mut money_text: Query<&mut Text, (With<MoneyText>, Without<WaveText>, Without<KillCountText>, Without<HealthText>)>,
    mut kill_text: Query<&mut Text, (With<KillCountText>, Without<WaveText>, Without<MoneyText>, Without<HealthText>)>,
    mut kill_bar: Query<&mut Node, (With<KillProgressBar>, Without<HealthBar>, Without<CameraAngleFill>, Without<SpriteTiltFill>)>,
    mut health_text: Query<&mut Text, (With<HealthText>, Without<WaveText>, Without<MoneyText>, Without<KillCountText>)>,
    mut health_bar: Query<&mut Node, (With<HealthBar>, Without<KillProgressBar>, Without<CameraAngleFill>, Without<SpriteTiltFill>)>,
) {
    if let Ok(mut text) = wave_text.single_mut() {
        **text = format!("Wave: {}", wave_state.current_wave);
    }

    if let Ok(mut text) = money_text.single_mut() {
        **text = format!("Coins: {}", money.get());
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
            .unwrap_or_default();

        if let Ok(mut text) = health_text.single_mut() {
            **text = format!("HP: {}/{}", health.current as i32, max_life as i32);
        }

        if let Ok(mut node) = health_bar.single_mut() {
            let progress = (health.current / max_life * 100.0).clamp(0.0, 100.0);
            node.width = Val::Percent(progress);
        }
    }
}
