use bevy::prelude::*;

use crate::actors::{Health, Player};
use crate::palette;
use crate::run::{wave_duration, BreatherTimer, RunState};
use crate::stats::{ComputedStats, Stat};
use crate::GameState;

use super::widgets::panel_node;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct CountdownText;

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
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            None,
        ),
        children![
            (
                WaveText,
                Text::new(format!("Wave {}", run_state.wave)),
                TextFont { font_size: 24.0, ..default() },
                TextColor(palette::color("ui_text_title")),
            ),
            (
                LifeText,
                Text::new("Life: 0/0"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(palette::color("ui_text")),
            ),
            (
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(24.0),
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(palette::color("ui_lifebar_bg")),
                BorderColor::all(palette::color("ui_panel_border")),
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

    commands.spawn((
        Name::new("CountdownText"),
        CountdownText,
        DespawnOnExit(GameState::Playing),
        Text::new(""),
        TextFont { font_size: 56.0, ..default() },
        TextColor(palette::color("ui_text_title")),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-40.0)),
            ..default()
        },
    ));
}

pub fn update_hud(
    run_state: Res<RunState>,
    breather: Option<Res<BreatherTimer>>,
    player_query: Query<(&Health, &ComputedStats), With<Player>>,
    mut wave_text: Query<&mut Text, (With<WaveText>, Without<LifeText>, Without<CountdownText>)>,
    mut life_text: Query<&mut Text, (With<LifeText>, Without<WaveText>, Without<CountdownText>)>,
    mut countdown_text: Query<&mut Text, (With<CountdownText>, Without<WaveText>, Without<LifeText>)>,
    mut life_bar: Query<&mut Node, With<LifeBar>>,
) {
    if let Ok(mut text) = wave_text.single_mut() {
        **text = format!("Wave {}", run_state.wave);
    }

    if let Ok(mut text) = countdown_text.single_mut() {
        let remaining = if let Some(b) = breather.as_ref() {
            b.0.remaining_secs()
        } else {
            let total = wave_duration(run_state.wave);
            (total - run_state.elapsed).max(0.0)
        };
        **text = format!("{}", remaining.ceil() as u32);
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
