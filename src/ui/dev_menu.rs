use bevy::prelude::*;
use bevy::time::Real;
use bevy::ui::UiGlobalTransform;

use crate::actors::Health;
use crate::actors::Player;
use crate::arena::{CameraAngle, CameraZoom};
use crate::game_state::GameState;
use crate::palette;
use crate::run::PlayerMoney;
use crate::stats::{DirtyStats, ModifierKind, Modifiers, Stat};
use crate::transition::{Transition, TransitionAction};
use crate::wave::EnemySpawnPool;
use crate::wave::{CombatPhase, WavePhase};

use super::widgets::{button_node, panel_node, ReleasedButtons};

const ANGLE_MIN: f32 = 1.0;
const ANGLE_MAX: f32 = 89.0;
const ZOOM_MIN: f32 = 500.0;
const ZOOM_MAX: f32 = 2500.0;

#[derive(Component)]
pub(super) struct CameraAngleSlider;

#[derive(Component)]
pub(super) struct AngleFill;

#[derive(Component)]
pub(super) struct AngleValueText;

#[derive(Component)]
pub(super) struct CameraZoomSlider;

#[derive(Component)]
pub(super) struct ZoomFill;

#[derive(Component)]
pub(super) struct ZoomValueText;

#[derive(Component)]
pub(super) struct CheatMoneyButton;

#[derive(Component)]
pub(super) struct CheatHealthButton;

#[derive(Component)]
pub(super) struct CheatDamageButton;

#[derive(Component)]
pub(super) struct CheatWinWaveButton;

#[derive(Component)]
pub(super) struct EnemyToggleButton(pub usize);

#[derive(Component)]
pub(super) struct EnableAllEnemiesButton;

#[derive(Component)]
pub(super) struct DisableAllEnemiesButton;

#[derive(Component)]
pub(super) struct EnemyToggleText(pub usize);

#[derive(Resource, Default)]
pub(super) struct ShopDevMenuOpen(pub bool);

#[derive(Component)]
pub(super) struct ShopDevMenuRoot;

pub(super) fn dev_menu_active(
    combat: Option<Res<State<CombatPhase>>>,
    shop: Res<ShopDevMenuOpen>,
) -> bool {
    let combat_open = combat
        .map(|s| *s.get() == CombatPhase::DevMenu)
        .unwrap_or(false);
    combat_open || shop.0
}

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

pub(super) fn toggle_shop_dev_menu(
    key: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<ShopDevMenuOpen>,
) {
    if key.just_pressed(KeyCode::Backquote) {
        open.0 = !open.0;
    } else if key.just_pressed(KeyCode::Escape) && open.0 {
        open.0 = false;
    }
}

pub(super) fn react_to_shop_dev_menu(
    mut commands: Commands,
    open: Res<ShopDevMenuOpen>,
    existing: Query<Entity, With<ShopDevMenuRoot>>,
    camera_angle: Res<CameraAngle>,
    camera_zoom: Res<CameraZoom>,
    spawn_pool: Res<EnemySpawnPool>,
) {
    if !open.is_changed() {
        return;
    }
    if open.0 {
        spawn_shop_dev_menu(&mut commands, &camera_angle, &camera_zoom, &spawn_pool);
    } else {
        for e in &existing {
            commands.entity(e).despawn();
        }
    }
}

pub(super) fn reset_shop_dev_menu(mut open: ResMut<ShopDevMenuOpen>) {
    open.0 = false;
}

fn spawn_shop_dev_menu(
    commands: &mut Commands,
    camera_angle: &CameraAngle,
    camera_zoom: &CameraZoom,
    spawn_pool: &EnemySpawnPool,
) {
    let panel = build_dev_menu_panel(commands, camera_angle, camera_zoom, spawn_pool, false);
    commands
        .spawn((
            Name::new("ShopDevMenuRoot"),
            ShopDevMenuRoot,
            DespawnOnExit(WavePhase::Shop),
            GlobalZIndex(100),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(palette::color_alpha("ui_overlay_bg", 0.6)),
        ))
        .add_child(panel);
}

fn cheat_button(label: &str, color: Color, marker: impl Component) -> impl Bundle {
    (
        marker,
        button_node(
            Node {
                margin: UiRect::top(Val::Px(10.0)),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        ),
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
        palette::color("ui_text_positive")
    } else {
        palette::color("ui_text_disabled")
    };
    let label = if enabled {
        format!("[x] {}", name)
    } else {
        format!("[  ] {}", name)
    };

    (
        EnemyToggleButton(index),
        button_node(
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                margin: UiRect::top(Val::Px(4.0)),
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            None,
        ),
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
    camera_zoom: Res<CameraZoom>,
    spawn_pool: Res<EnemySpawnPool>,
) {
    let panel = build_dev_menu_panel(&mut commands, &camera_angle, &camera_zoom, &spawn_pool, true);
    commands
        .spawn((
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
            BackgroundColor(palette::color_alpha("ui_overlay_bg", 0.6)),
        ))
        .add_child(panel);
}

fn build_dev_menu_panel(
    commands: &mut Commands,
    camera_angle: &CameraAngle,
    camera_zoom: &CameraZoom,
    spawn_pool: &EnemySpawnPool,
    include_win_wave: bool,
) -> Entity {
    let angle_t = (camera_angle.degrees - ANGLE_MIN) / (ANGLE_MAX - ANGLE_MIN);
    let zoom_t = (camera_zoom.height - ZOOM_MIN) / (ZOOM_MAX - ZOOM_MIN);

    let mut enemy_rows: Vec<Entity> = Vec::new();
    for (i, (kind, enabled)) in spawn_pool.enabled.iter().enumerate() {
        let row = commands.spawn(enemy_toggle_row(i, kind.id(), *enabled)).id();
        enemy_rows.push(row);
    }

    let enable_all = commands.spawn(cheat_button(
        "Enable All",
        palette::color("ui_text_positive"),
        EnableAllEnemiesButton,
    )).id();
    let disable_all = commands.spawn(cheat_button(
        "Disable All",
        palette::color("ui_text_negative"),
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
        TextColor(palette::color("ui_text_subtle")),
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
        TextColor(palette::color("ui_text_title")),
        Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
    )).id();

    let angle_label = commands.spawn((
        Text::new("Camera Angle"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(palette::color("ui_text_subtle")),
    )).id();
    let angle_value = commands.spawn((
        AngleValueText,
        Text::new(format!("{:.0}\u{00b0}", camera_angle.degrees)),
        TextFont { font_size: 20.0, ..default() },
        TextColor(palette::color("ui_text")),
    )).id();
    let angle_row = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
    )).add_children(&[angle_label, angle_value]).id();

    let angle_fill = commands.spawn((
        AngleFill,
        Node {
            width: Val::Percent(angle_t * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(palette::color("ui_button_pressed")),
    )).id();
    let angle_slider = commands.spawn((
        CameraAngleSlider,
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(palette::color("ui_button_normal")),
    )).add_child(angle_fill).id();

    let zoom_label = commands.spawn((
        Text::new("Camera View"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(palette::color("ui_text_subtle")),
    )).id();
    let zoom_value = commands.spawn((
        ZoomValueText,
        Text::new(format!("{:.0}", camera_zoom.height)),
        TextFont { font_size: 20.0, ..default() },
        TextColor(palette::color("ui_text")),
    )).id();
    let zoom_row = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(12.0), Val::Px(8.0)),
            ..default()
        },
    )).add_children(&[zoom_label, zoom_value]).id();

    let zoom_fill = commands.spawn((
        ZoomFill,
        Node {
            width: Val::Percent(zoom_t * 100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(palette::color("ui_button_pressed")),
    )).id();
    let zoom_slider = commands.spawn((
        CameraZoomSlider,
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(palette::color("ui_button_normal")),
    )).add_child(zoom_fill).id();

    let money_btn = commands.spawn(cheat_button("Money +1000", palette::color("ui_text_money"), CheatMoneyButton)).id();
    let health_btn = commands.spawn(cheat_button("Health +100", palette::color("ui_text_positive"), CheatHealthButton)).id();
    let damage_btn = commands.spawn(cheat_button("Phys Damage +100", palette::color("ui_text_negative"), CheatDamageButton)).id();

    let mut children = vec![
        title,
        angle_row,
        angle_slider,
        zoom_row,
        zoom_slider,
        money_btn,
        health_btn,
        damage_btn,
    ];
    if include_win_wave {
        let win_wave_btn = commands.spawn(cheat_button("Win Wave", palette::color("ui_text"), CheatWinWaveButton)).id();
        children.push(win_wave_btn);
    }
    children.push(enemy_container);

    commands.spawn(panel_node(
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::all(Val::Px(40.0)),
            width: Val::Px(500.0),
            max_height: Val::Percent(90.0),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    )).add_children(&children).id()
}

pub(super) fn cheat_money(
    buttons: ReleasedButtons<CheatMoneyButton>,
    mut money: ResMut<PlayerMoney>,
) {
    buttons.for_each(|_| {
        money.earn(1000);
    });
}

pub(super) fn cheat_health(
    buttons: ReleasedButtons<CheatHealthButton>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats, &mut Health), With<Player>>,
) {
    buttons.for_each(|_| {
        for (mut modifiers, mut dirty, mut health) in &mut player_query {
            modifiers.add(Stat::MaxLife, ModifierKind::Flat, 100.0);
            dirty.mark(Stat::MaxLife);
            health.current += 100.0;
        }
    });
}

pub(super) fn cheat_damage(
    buttons: ReleasedButtons<CheatDamageButton>,
    mut player_query: Query<(&mut Modifiers, &mut DirtyStats), With<Player>>,
) {
    buttons.for_each(|_| {
        for (mut modifiers, mut dirty) in &mut player_query {
            modifiers.add(Stat::PhysicalDamage, ModifierKind::Flat, 100.0);
            dirty.mark(Stat::PhysicalDamage);
        }
    });
}

pub(super) fn cheat_win_wave(
    buttons: ReleasedButtons<CheatWinWaveButton>,
    mut next_combat: ResMut<NextState<CombatPhase>>,
    mut transition: ResMut<Transition>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    buttons.for_each(|_| {
        virtual_time.unpause();
        next_combat.set(CombatPhase::Running);
        transition.request(TransitionAction::Wave(WavePhase::Shop));
    });
}

pub(super) fn toggle_enemy_type(
    buttons: ReleasedButtons<EnemyToggleButton>,
    mut spawn_pool: ResMut<EnemySpawnPool>,
    mut text_query: Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
) {
    buttons.for_each(|toggle| {
        if let Some((kind, enabled)) = spawn_pool.enabled.get_mut(toggle.0) {
            *enabled = !*enabled;
            let new_enabled = *enabled;
            let name = kind.id();
            update_toggle_text(&mut text_query, toggle.0, name, new_enabled);
        }
    });
}

pub(super) fn enable_all_enemies(
    buttons: ReleasedButtons<EnableAllEnemiesButton>,
    mut spawn_pool: ResMut<EnemySpawnPool>,
    mut text_query: Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
) {
    buttons.for_each(|_| {
        for i in 0..spawn_pool.enabled.len() {
            spawn_pool.enabled[i].1 = true;
            let name = spawn_pool.enabled[i].0.id();
            update_toggle_text(&mut text_query, i, name, true);
        }
    });
}

pub(super) fn disable_all_enemies(
    buttons: ReleasedButtons<DisableAllEnemiesButton>,
    mut spawn_pool: ResMut<EnemySpawnPool>,
    mut text_query: Query<(&EnemyToggleText, &mut Text, &mut TextColor)>,
) {
    buttons.for_each(|_| {
        for i in 0..spawn_pool.enabled.len() {
            spawn_pool.enabled[i].1 = false;
            let name = spawn_pool.enabled[i].0.id();
            update_toggle_text(&mut text_query, i, name, false);
        }
    });
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
                TextColor(palette::color("ui_text_positive"))
            } else {
                TextColor(palette::color("ui_text_disabled"))
            };
        }
    }
}

fn slider_drag_t(
    interaction: &Interaction,
    ui_transform: &UiGlobalTransform,
    computed: &ComputedNode,
    window: &Window,
    mouse: &ButtonInput<MouseButton>,
) -> Option<f32> {
    let active = *interaction == Interaction::Pressed
        || (*interaction == Interaction::Hovered && mouse.pressed(MouseButton::Left));
    if !active {
        return None;
    }
    let cursor_pos = window.cursor_position()?;
    let inverse = ui_transform.try_inverse()?;
    let local = inverse.transform_point2(cursor_pos * window.scale_factor());
    let node_size = computed.size();
    Some(((local.x / node_size.x) + 0.5).clamp(0.0, 1.0))
}

pub(super) fn camera_angle_slider_interaction(
    track_query: Query<(&Interaction, &UiGlobalTransform, &ComputedNode), With<CameraAngleSlider>>,
    mut fill_query: Query<&mut Node, With<AngleFill>>,
    mut text_query: Query<&mut Text, With<AngleValueText>>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_angle: ResMut<CameraAngle>,
) {
    let Ok(window) = windows.single() else { return };
    for (interaction, ui_transform, computed) in &track_query {
        let Some(t) = slider_drag_t(interaction, ui_transform, computed, window, &mouse) else {
            continue;
        };
        let value = ANGLE_MIN + t * (ANGLE_MAX - ANGLE_MIN);
        camera_angle.degrees = value;
        for mut node in &mut fill_query {
            node.width = Val::Percent(t * 100.0);
        }
        for mut text in &mut text_query {
            *text = Text::new(format!("{:.0}\u{00b0}", value));
        }
    }
}

pub(super) fn camera_zoom_slider_interaction(
    track_query: Query<(&Interaction, &UiGlobalTransform, &ComputedNode), With<CameraZoomSlider>>,
    mut fill_query: Query<&mut Node, With<ZoomFill>>,
    mut text_query: Query<&mut Text, With<ZoomValueText>>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera_zoom: ResMut<CameraZoom>,
) {
    let Ok(window) = windows.single() else { return };
    for (interaction, ui_transform, computed) in &track_query {
        let Some(t) = slider_drag_t(interaction, ui_transform, computed, window, &mouse) else {
            continue;
        };
        let value = ZOOM_MIN + t * (ZOOM_MAX - ZOOM_MIN);
        camera_zoom.height = value;
        for mut node in &mut fill_query {
            node.width = Val::Percent(t * 100.0);
        }
        for mut text in &mut text_query {
            *text = Text::new(format!("{:.0}", value));
        }
    }
}

#[derive(Component)]
pub(super) struct DevTrigger;

#[derive(Resource, Default)]
pub(super) struct DevTriggerState {
    last_tap_secs: f32,
}

const DEV_TRIGGER_DOUBLE_TAP_WINDOW: f32 = 0.5;

pub(super) fn spawn_dev_trigger(mut commands: Commands) {
    let size = 140.0;
    commands.spawn((
        Name::new("DevTrigger"),
        DevTrigger,
        Button,
        DespawnOnExit(GameState::Playing),
        GlobalZIndex(50),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Percent(50.0),
            width: Val::Px(size),
            height: Val::Px(size),
            margin: UiRect {
                left: Val::Px(-size / 2.0),
                ..UiRect::ZERO
            },
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
}

pub(super) fn dev_trigger_double_tap(
    query: Query<&Interaction, (Changed<Interaction>, With<DevTrigger>)>,
    mut state: ResMut<DevTriggerState>,
    time: Res<Time<Real>>,
    wave_phase: Option<Res<State<WavePhase>>>,
    combat_phase: Option<Res<State<CombatPhase>>>,
    next_combat: Option<ResMut<NextState<CombatPhase>>>,
    shop_open: Option<ResMut<ShopDevMenuOpen>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    let Some(&interaction) = query.iter().next() else {
        return;
    };
    if interaction != Interaction::Pressed {
        return;
    }
    let now = time.elapsed_secs();
    if now - state.last_tap_secs > DEV_TRIGGER_DOUBLE_TAP_WINDOW {
        state.last_tap_secs = now;
        return;
    }
    state.last_tap_secs = 0.0;

    let Some(wave_phase) = wave_phase else {
        return;
    };
    match wave_phase.get() {
        WavePhase::Combat => {
            let (Some(combat), Some(mut next)) = (combat_phase, next_combat) else {
                return;
            };
            match combat.get() {
                CombatPhase::Running | CombatPhase::Paused => {
                    virtual_time.pause();
                    next.set(CombatPhase::DevMenu);
                }
                CombatPhase::DevMenu => {
                    virtual_time.unpause();
                    next.set(CombatPhase::Running);
                }
            }
        }
        WavePhase::Shop => {
            if let Some(mut shop_open) = shop_open {
                shop_open.0 = !shop_open.0;
            }
        }
    }
}
