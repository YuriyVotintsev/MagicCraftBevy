use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use crate::arena::{CameraAngle, EnemySpawnPool};
use crate::actors::components::common::health::Health;
use crate::money::PlayerMoney;
use crate::player::Player;
use crate::stats::{DirtyStats, Modifiers, Stat};
use crate::wave::CombatPhase;

const SLIDER_MIN: f32 = 1.0;
const SLIDER_MAX: f32 = 89.0;

#[derive(Component)]
pub(super) struct CameraAngleSlider;

#[derive(Component)]
pub(super) struct SliderFill;

#[derive(Component)]
pub(super) struct SliderValueText;

#[derive(Component)]
pub(super) struct CheatMoneyButton;

#[derive(Component)]
pub(super) struct CheatHealthButton;

#[derive(Component)]
pub(super) struct CheatDamageButton;

#[derive(Component)]
pub(super) struct EnemyToggleButton(pub usize);

#[derive(Component)]
pub(super) struct EnableAllEnemiesButton;

#[derive(Component)]
pub(super) struct DisableAllEnemiesButton;

#[derive(Component)]
pub(super) struct EnemyToggleText(pub usize);

pub(super) fn toggle_dev_menu(
    key: Res<ButtonInput<KeyCode>>,
    combat_phase: Res<State<CombatPhase>>,
    mut next_phase: ResMut<NextState<CombatPhase>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    if key.just_pressed(KeyCode::Backquote) {
        match combat_phase.get() {
            CombatPhase::Running => {
                virtual_time.pause();
                next_phase.set(CombatPhase::DevMenu);
            }
            CombatPhase::DevMenu => {
                virtual_time.unpause();
                next_phase.set(CombatPhase::Running);
            }
            CombatPhase::Paused => {
                next_phase.set(CombatPhase::DevMenu);
            }
        }
    }
    if key.just_pressed(KeyCode::Escape) && *combat_phase.get() == CombatPhase::DevMenu {
        virtual_time.unpause();
        next_phase.set(CombatPhase::Running);
    }
}

fn cheat_button(label: &str, color: Color, marker: impl Component) -> impl Bundle {
    (
        marker,
        Button,
        Node {
            margin: UiRect::top(Val::Px(10.0)),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
        children![
            (
                Text::new(label),
                TextFont { font_size: 22.0, ..default() },
                TextColor(color)
            )
        ],
    )
}

fn enemy_toggle_row(index: usize, name: &str, enabled: bool) -> impl Bundle {
    let text_color = if enabled {
        Color::srgb(0.3, 0.9, 0.3)
    } else {
        Color::srgb(0.5, 0.5, 0.5)
    };
    let label = if enabled {
        format!("[x] {}", name)
    } else {
        format!("[  ] {}", name)
    };

    (
        EnemyToggleButton(index),
        Button,
        Node {
            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
            margin: UiRect::top(Val::Px(4.0)),
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        children![
            (
                EnemyToggleText(index),
                Text::new(label),
                TextFont { font_size: 20.0, ..default() },
                TextColor(text_color)
            )
        ],
    )
}

pub(super) fn spawn_dev_menu(
    mut commands: Commands,
    camera_angle: Res<CameraAngle>,
    spawn_pool: Res<EnemySpawnPool>,
) {
    let t = (camera_angle.degrees - SLIDER_MIN) / (SLIDER_MAX - SLIDER_MIN);

    let mut enemy_rows: Vec<Entity> = Vec::new();
    for (i, (kind, enabled)) in spawn_pool.enabled.iter().enumerate() {
        let row = commands.spawn(enemy_toggle_row(i, kind.id(), *enabled)).id();
        enemy_rows.push(row);
    }

    let enable_all = commands.spawn(cheat_button(
        "Enable All",
        Color::srgb(0.3, 0.9, 0.3),
        EnableAllEnemiesButton,
    )).id();
    let disable_all = commands.spawn(cheat_button(
        "Disable All",
        Color::srgb(0.9, 0.3, 0.3),
        DisableAllEnemiesButton,
    )).id();

    let bulk_row = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
    )).add_children(&[enable_all, disable_all]).id();

    let mut enemy_section_children: Vec<Entity> = Vec::new();
    let section_label = commands.spawn((
        Text::new("Enemy Spawn Pool"),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
    )).id();
    enemy_section_children.push(section_label);
    enemy_section_children.extend(enemy_rows);
    enemy_section_children.push(bulk_row);

    let enemy_container = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
    )).add_children(&enemy_section_children).id();

    let title = commands.spawn((
        Text::new("Dev Menu"),
        TextFont { font_size: 48.0, ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.9)),
        Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
    )).id();

    let angle_label = commands.spawn((
        Text::new("Camera Angle"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
    )).id();
    let angle_value = commands.spawn((
        SliderValueText,
        Text::new(format!("{:.0}\u{00b0}", camera_angle.degrees)),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    )).id();
    let angle_row = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
    )).add_children(&[angle_label, angle_value]).id();

    let slider_fill = commands.spawn((
        SliderFill,
        Node {
            width: Val::Percent(t * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.6, 0.9)),
    )).id();
    let slider = commands.spawn((
        CameraAngleSlider,
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
    )).add_child(slider_fill).id();

    let money_btn = commands.spawn(cheat_button("Money +1000", Color::srgb(0.9, 0.85, 0.3), CheatMoneyButton)).id();
    let health_btn = commands.spawn(cheat_button("Health +100", Color::srgb(0.3, 0.9, 0.4), CheatHealthButton)).id();
    let damage_btn = commands.spawn(cheat_button("Phys Damage +100", Color::srgb(0.9, 0.4, 0.3), CheatDamageButton)).id();

    let panel = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::all(Val::Px(40.0)),
            width: Val::Px(500.0),
            max_height: Val::Percent(90.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
    )).add_children(&[
        title, angle_row, slider, money_btn, health_btn, damage_btn, enemy_container,
    ]).id();

    commands.spawn((
        Name::new("DevMenuRoot"),
        DespawnOnExit(CombatPhase::DevMenu),
        GlobalZIndex(100),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
    )).add_child(panel);
}

pub(super) fn cheat_money(
    query: Query<&Interaction, (Changed<Interaction>, With<CheatMoneyButton>)>,
    mut money: ResMut<PlayerMoney>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            money.earn(1000);
        }
    }
}

pub(super) fn cheat_health(
    query: Query<&Interaction, (Changed<Interaction>, With<CheatHealthButton>)>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats, &mut Health), With<Player>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for (mut modifiers, mut dirty, mut health) in &mut player_query {
                modifiers.add(Stat::MaxLifeFlat, 100.0);
                dirty.mark(Stat::MaxLifeFlat);
                health.current += 100.0;
            }
        }
    }
}

pub(super) fn cheat_damage(
    query: Query<&Interaction, (Changed<Interaction>, With<CheatDamageButton>)>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats), With<Player>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for (mut modifiers, mut dirty) in &mut player_query {
                modifiers.add(Stat::PhysicalDamageFlat, 100.0);
                dirty.mark(Stat::PhysicalDamageFlat);
            }
        }
    }
}

pub(super) fn toggle_enemy_type(
    query: Query<(&Interaction, &EnemyToggleButton), Changed<Interaction>>,
    mut spawn_pool: ResMut<EnemySpawnPool>,
    mut text_query: Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
) {
    for (interaction, toggle) in &query {
        if *interaction == Interaction::Pressed {
            if let Some((kind, enabled)) = spawn_pool.enabled.get_mut(toggle.0) {
                *enabled = !*enabled;
                let new_enabled = *enabled;
                let name = kind.id();
                update_toggle_text(&mut text_query, toggle.0, name, new_enabled);
            }
        }
    }
}

pub(super) fn enable_all_enemies(
    query: Query<&Interaction, (Changed<Interaction>, With<EnableAllEnemiesButton>)>,
    mut spawn_pool: ResMut<EnemySpawnPool>,
    mut text_query: Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for i in 0..spawn_pool.enabled.len() {
                spawn_pool.enabled[i].1 = true;
                let name = spawn_pool.enabled[i].0.id();
                update_toggle_text(&mut text_query, i, name, true);
            }
        }
    }
}

pub(super) fn disable_all_enemies(
    query: Query<&Interaction, (Changed<Interaction>, With<DisableAllEnemiesButton>)>,
    mut spawn_pool: ResMut<EnemySpawnPool>,
    mut text_query: Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            for i in 0..spawn_pool.enabled.len() {
                spawn_pool.enabled[i].1 = false;
                let name = spawn_pool.enabled[i].0.id();
                update_toggle_text(&mut text_query, i, name, false);
            }
        }
    }
}

fn update_toggle_text(
    text_query: &mut Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
    index: usize,
    name: &str,
    enabled: bool,
) {
    for (toggle_text, mut text, mut color) in text_query.iter_mut() {
        if toggle_text.0 == index {
            let label = if enabled {
                format!("[x] {}", name)
            } else {
                format!("[  ] {}", name)
            };
            *text = Text::new(label);
            *color = if enabled {
                TextColor(Color::srgb(0.3, 0.9, 0.3))
            } else {
                TextColor(Color::srgb(0.5, 0.5, 0.5))
            };
        }
    }
}

pub(super) fn slider_interaction(
    track_query: Query<(&Interaction, &UiGlobalTransform, &ComputedNode), With<CameraAngleSlider>>,
    mut fill_query: Query<&mut Node, With<SliderFill>>,
    mut text_query: Query<&mut Text, With<SliderValueText>>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_angle: ResMut<CameraAngle>,
) {
    for (interaction, ui_transform, computed) in &track_query {
        let active = *interaction == Interaction::Pressed
            || (*interaction == Interaction::Hovered && mouse.pressed(MouseButton::Left));
        if !active {
            continue;
        }
        let Ok(window) = windows.single() else { continue };
        let Some(cursor_pos) = window.cursor_position() else { continue };

        let Some(inverse) = ui_transform.try_inverse() else { continue };
        let local = inverse.transform_point2(cursor_pos * window.scale_factor());
        let node_size = computed.size();
        let t = ((local.x / node_size.x) + 0.5).clamp(0.0, 1.0);
        let value = SLIDER_MIN + t * (SLIDER_MAX - SLIDER_MIN);

        camera_angle.degrees = value;

        for mut node in &mut fill_query {
            node.width = Val::Percent(t * 100.0);
        }
        for mut text in &mut text_query {
            *text = Text::new(format!("{:.0}\u{00b0}", value));
        }
    }
}
