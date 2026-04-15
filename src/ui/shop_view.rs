use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::PrimaryWindow;

use crate::run::PlayerMoney;
use crate::run::RunState;
use crate::rune::{
    DragState, GridCellView, HexCoord, JokerSlotView, JokerSlots, RuneGrid, RuneSource,
    RuneStub, RuneView, ShopOffer, ShopSlotView, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
use crate::wave::WavePhase;

const BG_COLOR: Color = Color::srgb(0.03, 0.03, 0.08);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const CELL_UNLOCKED_BG: Color = Color::srgba(0.15, 0.15, 0.22, 0.85);
const CELL_UNLOCKED_BORDER: Color = Color::srgba(0.45, 0.45, 0.55, 0.9);
const CELL_LOCKED_BG: Color = Color::srgba(0.08, 0.08, 0.10, 0.5);
const CELL_LOCKED_BORDER: Color = Color::srgba(0.25, 0.25, 0.30, 0.5);

const JOKER_BG: Color = Color::srgba(0.20, 0.10, 0.25, 0.75);
const JOKER_BORDER: Color = Color::srgba(1.0, 0.78, 0.25, 0.85);

const SHOP_SLOT_BG: Color = Color::srgba(0.10, 0.12, 0.18, 0.85);
const SHOP_SLOT_BORDER: Color = Color::srgba(0.35, 0.40, 0.50, 0.9);
const SHOP_PANEL_BG: Color = Color::srgba(0.06, 0.08, 0.12, 0.9);

const RUNE_BORDER: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);

const GRID_CENTER: Vec2 = Vec2::new(860.0, 520.0);
const CELL_SIZE: f32 = 40.0;
const CELL_DIAMETER: f32 = 64.0;
const RUNE_DIAMETER: f32 = 48.0;

const SHOP_PANEL_X: f32 = 1650.0;
const SHOP_PANEL_Y: f32 = 120.0;
const SHOP_PANEL_W: f32 = 220.0;
const SHOP_PANEL_H: f32 = 620.0;
const SHOP_SLOT_Y0: f32 = 180.0;
const SHOP_SLOT_STEP: f32 = 130.0;
const SHOP_SLOT_X: f32 = SHOP_PANEL_X + SHOP_PANEL_W * 0.5;

const JOKER_DIAMETER: f32 = 60.0;
const JOKER_POSITIONS: [Vec2; JOKER_SLOTS] = [
    Vec2::new(0.0, -270.0),
    Vec2::new(234.0, -135.0),
    Vec2::new(234.0, 135.0),
    Vec2::new(0.0, 270.0),
    Vec2::new(-234.0, 135.0),
    Vec2::new(-234.0, -135.0),
];

fn grid_cell_center(coord: HexCoord) -> Vec2 {
    GRID_CENTER + coord.to_pixel(CELL_SIZE)
}

fn shop_slot_center(idx: usize) -> Vec2 {
    Vec2::new(SHOP_SLOT_X, SHOP_SLOT_Y0 + idx as f32 * SHOP_SLOT_STEP)
}

fn joker_slot_center(idx: usize) -> Vec2 {
    GRID_CENTER + JOKER_POSITIONS[idx]
}

fn home_position(source: RuneSource) -> Vec2 {
    match source {
        RuneSource::Shop(idx) => shop_slot_center(idx),
        RuneSource::Grid(coord) => grid_cell_center(coord),
    }
}

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct ShopRoot;

pub fn spawn_shop_screen(
    mut commands: Commands,
    run_state: Res<RunState>,
    money: Res<PlayerMoney>,
    shop: Res<ShopOffer>,
    grid: Res<RuneGrid>,
    _jokers: Res<JokerSlots>,
) {
    let root = commands
        .spawn((
            Name::new("ShopRoot"),
            ShopRoot,
            DespawnOnExit(WavePhase::Shop),
            GlobalZIndex(50),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(BG_COLOR),
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        Text(format!("Run {}", run_state.attempt)),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(40.0),
            top: Val::Px(24.0),
            ..default()
        },
    ));

    commands.spawn((
        ChildOf(root),
        ShopCoinsText,
        Text(format!("Coins: {}", money.get())),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(GOLD_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(40.0),
            top: Val::Px(60.0),
            ..default()
        },
    ));

    commands.spawn((
        ChildOf(root),
        Button,
        StartRunButton,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(1710.0),
            top: Val::Px(24.0),
            width: Val::Px(170.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        children![(
            Text::new("Start Run"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(TEXT_COLOR),
        )],
    ));

    commands.spawn((
        ChildOf(root),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(SHOP_PANEL_X),
            top: Val::Px(SHOP_PANEL_Y),
            width: Val::Px(SHOP_PANEL_W),
            height: Val::Px(SHOP_PANEL_H),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(SHOP_PANEL_BG),
        BorderColor::all(SHOP_SLOT_BORDER),
    ));

    commands.spawn((
        ChildOf(root),
        Text::new("SHOP"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(SHOP_PANEL_X + 78.0),
            top: Val::Px(SHOP_PANEL_Y + 16.0),
            ..default()
        },
    ));

    for coord in HexCoord::all_within_radius(GRID_RADIUS) {
        let center = grid_cell_center(coord);
        let unlocked = grid.is_unlocked(coord);
        let (bg, border) = if unlocked {
            (CELL_UNLOCKED_BG, CELL_UNLOCKED_BORDER)
        } else {
            (CELL_LOCKED_BG, CELL_LOCKED_BORDER)
        };
        commands.spawn((
            ChildOf(root),
            GridCellView {
                coord,
                center,
                unlocked,
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - CELL_DIAMETER * 0.5),
                top: Val::Px(center.y - CELL_DIAMETER * 0.5),
                width: Val::Px(CELL_DIAMETER),
                height: Val::Px(CELL_DIAMETER),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border),
        ));
    }

    for idx in 0..JOKER_SLOTS {
        let center = joker_slot_center(idx);
        commands.spawn((
            ChildOf(root),
            JokerSlotView { index: idx, center },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - JOKER_DIAMETER * 0.5),
                top: Val::Px(center.y - JOKER_DIAMETER * 0.5),
                width: Val::Px(JOKER_DIAMETER),
                height: Val::Px(JOKER_DIAMETER),
                border: UiRect::all(Val::Px(3.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(JOKER_BG),
            BorderColor::all(JOKER_BORDER),
        ));
    }

    for idx in 0..SHOP_SLOTS {
        let center = shop_slot_center(idx);
        commands.spawn((
            ChildOf(root),
            ShopSlotView { index: idx, center },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - CELL_DIAMETER * 0.5),
                top: Val::Px(center.y - CELL_DIAMETER * 0.5),
                width: Val::Px(CELL_DIAMETER),
                height: Val::Px(CELL_DIAMETER),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(SHOP_SLOT_BG),
            BorderColor::all(SHOP_SLOT_BORDER),
        ));
    }

    for (idx, slot) in shop.runes.iter().enumerate() {
        if let Some(stub) = slot {
            spawn_rune_entity(&mut commands, root, *stub, RuneSource::Shop(idx));
        }
    }
    for (coord, stub) in grid.cells.iter() {
        spawn_rune_entity(&mut commands, root, *stub, RuneSource::Grid(*coord));
    }
}

fn spawn_rune_entity(
    commands: &mut Commands,
    root: Entity,
    stub: RuneStub,
    source: RuneSource,
) {
    let center = home_position(source);
    commands.spawn((
        ChildOf(root),
        RuneView { source },
        GlobalZIndex(60),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(center.x - RUNE_DIAMETER * 0.5),
            top: Val::Px(center.y - RUNE_DIAMETER * 0.5),
            width: Val::Px(RUNE_DIAMETER),
            height: Val::Px(RUNE_DIAMETER),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(stub.color),
        BorderColor::all(RUNE_BORDER),
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

fn cursor_ui_pos(window: &Window, ui_scale: f32) -> Option<Vec2> {
    let scale = if ui_scale > 0.0 { ui_scale } else { 1.0 };
    window.cursor_position().map(|c| c / scale)
}

pub fn start_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut drag: ResMut<DragState>,
    runes: Query<(Entity, &RuneView)>,
) {
    if !matches!(*drag, DragState::Idle) {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor) = cursor_ui_pos(window, ui_scale.0) else {
        return;
    };
    let radius = RUNE_DIAMETER * 0.5;
    for (entity, view) in &runes {
        let home = home_position(view.source);
        if home.distance(cursor) <= radius {
            *drag = DragState::Dragging {
                entity,
                from: view.source,
            };
            return;
        }
    }
}

pub fn follow_cursor(
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    drag: Res<DragState>,
    mut runes: Query<(Entity, &RuneView, &mut Node)>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let cursor = cursor_ui_pos(window, ui_scale.0);
    for (entity, view, mut node) in &mut runes {
        let pos = if let DragState::Dragging { entity: e, .. } = *drag {
            if e == entity {
                cursor.unwrap_or_else(|| home_position(view.source))
            } else {
                home_position(view.source)
            }
        } else {
            home_position(view.source)
        };
        node.left = Val::Px(pos.x - RUNE_DIAMETER * 0.5);
        node.top = Val::Px(pos.y - RUNE_DIAMETER * 0.5);
    }
}

fn find_drop_target(cursor: Vec2, grid: &RuneGrid) -> Option<RuneSource> {
    let cell_r = CELL_DIAMETER * 0.5;
    for coord in HexCoord::all_within_radius(GRID_RADIUS) {
        if !grid.is_unlocked(coord) {
            continue;
        }
        let center = grid_cell_center(coord);
        if center.distance(cursor) <= cell_r {
            return Some(RuneSource::Grid(coord));
        }
    }
    for idx in 0..SHOP_SLOTS {
        let center = shop_slot_center(idx);
        if center.distance(cursor) <= cell_r {
            return Some(RuneSource::Shop(idx));
        }
    }
    None
}

fn take_stub(src: RuneSource, shop: &mut ShopOffer, grid: &mut RuneGrid) -> Option<RuneStub> {
    match src {
        RuneSource::Shop(idx) => shop.runes[idx].take(),
        RuneSource::Grid(c) => grid.cells.remove(&c),
    }
}

fn place_stub(src: RuneSource, stub: RuneStub, shop: &mut ShopOffer, grid: &mut RuneGrid) {
    match src {
        RuneSource::Shop(idx) => shop.runes[idx] = Some(stub),
        RuneSource::Grid(c) => {
            grid.cells.insert(c, stub);
        }
    }
}

pub fn finish_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut drag: ResMut<DragState>,
    mut shop: ResMut<ShopOffer>,
    mut grid: ResMut<RuneGrid>,
    mut runes: Query<(Entity, &mut RuneView)>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let DragState::Dragging { entity, from } = *drag else {
        return;
    };
    *drag = DragState::Idle;

    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor) = cursor_ui_pos(window, ui_scale.0) else {
        return;
    };

    let Some(target) = find_drop_target(cursor, &grid) else {
        return;
    };
    if target == from {
        return;
    }

    let Some(dragged_stub) = take_stub(from, &mut shop, &mut grid) else {
        return;
    };
    let displaced_stub = take_stub(target, &mut shop, &mut grid);

    place_stub(target, dragged_stub, &mut shop, &mut grid);

    if let Some(other_entity) = runes
        .iter()
        .find(|(e, v)| *e != entity && v.source == target)
        .map(|(e, _)| e)
    {
        if let Ok((_, mut v)) = runes.get_mut(other_entity) {
            v.source = from;
        }
    }

    if let Some(stub) = displaced_stub {
        place_stub(from, stub, &mut shop, &mut grid);
    }

    if let Ok((_, mut v)) = runes.get_mut(entity) {
        v.source = target;
    }
}
