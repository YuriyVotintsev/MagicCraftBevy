use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::PrimaryWindow;

use crate::palette;
use crate::run::PlayerMoney;
use crate::run::RunState;
use crate::rune::{
    can_place, Dragging, GridCellView, HexCoord, JokerSlotView, JokerSlots, RuneGrid, RuneSource,
    RuneStub, RuneView, ShopOffer, ShopSlotView, StubKind, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
use crate::transition::{Transition, TransitionAction};
use crate::wave::WavePhase;

use super::panel_radius;

const RUNE_BORDER_WIDTH: f32 = 2.0;
const JOKER_BORDER_WIDTH: f32 = 10.0;

const GRID_CENTER: Vec2 = Vec2::new(820.0, 540.0);
const CELL_SIZE: f32 = 60.0;
const CELL_DIAMETER: f32 = 96.0;
const RUNE_DIAMETER: f32 = 72.0;

const SHOP_PANEL_X: f32 = 1620.0;
const SHOP_PANEL_Y: f32 = 110.0;
const SHOP_PANEL_W: f32 = 260.0;
const SHOP_PANEL_H: f32 = 650.0;
const SHOP_SLOT_Y0: f32 = 240.0;
const SHOP_SLOT_STEP: f32 = 130.0;
const SHOP_SLOT_X: f32 = SHOP_PANEL_X + SHOP_PANEL_W * 0.5;

const JOKER_DIAMETER: f32 = 90.0;
const JOKER_POSITIONS: [Vec2; JOKER_SLOTS] = [
    Vec2::new(0.0, -405.0),
    Vec2::new(351.0, -202.5),
    Vec2::new(351.0, 202.5),
    Vec2::new(0.0, 405.0),
    Vec2::new(-351.0, 202.5),
    Vec2::new(-351.0, -202.5),
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
        RuneSource::Joker(idx) => joker_slot_center(idx),
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
            BackgroundColor(palette::color("ui_screen_bg")),
        ))
        .id();

    commands.spawn((
        ChildOf(root),
        Text(format!("Run {}", run_state.attempt)),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(palette::color("ui_text")),
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
        TextColor(palette::color("ui_text_money")),
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
            border_radius: panel_radius(),
            ..default()
        },
        BackgroundColor(palette::color("ui_button_normal")),
        children![(
            Text::new("Start Run"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(palette::color("ui_text")),
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
            border_radius: panel_radius(),
            ..default()
        },
        BackgroundColor(palette::color("ui_panel_bg")),
        BorderColor::all(palette::color("ui_shop_slot_edge")),
    ));

    commands.spawn((
        ChildOf(root),
        Text::new("SHOP"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(palette::color("ui_text_title")),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(SHOP_PANEL_X + SHOP_PANEL_W * 0.5 - 30.0),
            top: Val::Px(SHOP_PANEL_Y + 24.0),
            ..default()
        },
    ));

    for coord in HexCoord::all_within_radius(GRID_RADIUS) {
        let center = grid_cell_center(coord);
        commands.spawn((
            ChildOf(root),
            GridCellView { coord },
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
            BackgroundColor(palette::color("ui_cell_locked_bg")),
            BorderColor::all(palette::color("ui_cell_locked_edge")),
        ));
    }

    for idx in 0..JOKER_SLOTS {
        let center = joker_slot_center(idx);
        commands.spawn((
            ChildOf(root),
            JokerSlotView { index: idx },
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
            BackgroundColor(palette::color("ui_joker_slot_bg")),
            BorderColor::all(palette::color("ui_joker_slot_edge")),
        ));
    }

    for idx in 0..SHOP_SLOTS {
        let center = shop_slot_center(idx);
        commands.spawn((
            ChildOf(root),
            ShopSlotView { index: idx },
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
            BackgroundColor(palette::color("ui_shop_slot_bg")),
            BorderColor::all(palette::color("ui_shop_slot_edge")),
        ));
    }
}

fn spawn_rune_entity(
    commands: &mut Commands,
    root: Entity,
    stub: RuneStub,
    source: RuneSource,
) {
    let center = home_position(source);
    let (border_width, border_color) = match stub.kind {
        StubKind::Rune => (RUNE_BORDER_WIDTH, palette::color("ui_rune_border")),
        StubKind::Joker => (JOKER_BORDER_WIDTH, palette::color("ui_joker_border")),
    };
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
            border: UiRect::all(Val::Px(border_width)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(stub.color),
        BorderColor::all(border_color),
    ));
}

pub fn start_run_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartRunButton>),
    >,
    mut transition: ResMut<Transition>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                *color = palette::color("ui_button_pressed").into();
                transition.request(TransitionAction::Wave(WavePhase::Combat));
            }
            Interaction::Hovered => *color = palette::color("ui_button_hover").into(),
            Interaction::None => *color = palette::color("ui_button_normal").into(),
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

pub fn sync_cell_lock_visuals(
    grid: Res<RuneGrid>,
    mut cells: Query<(&GridCellView, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (cell, mut bg, mut border) in &mut cells {
        let (bg_alias, border_alias) = if grid.is_unlocked(cell.coord) {
            ("ui_cell_unlocked_bg", "ui_cell_unlocked_edge")
        } else {
            ("ui_cell_locked_bg", "ui_cell_locked_edge")
        };
        *bg = BackgroundColor(palette::color(bg_alias));
        *border = BorderColor::all(palette::color(border_alias));
    }
}

pub fn reconcile_rune_entities(
    mut commands: Commands,
    root: Query<Entity, With<ShopRoot>>,
    shop: Res<ShopOffer>,
    grid: Res<RuneGrid>,
    jokers: Res<JokerSlots>,
    existing: Query<(Entity, &RuneView), Without<Dragging>>,
) {
    let Ok(root_entity) = root.single() else {
        return;
    };

    let mut by_source: std::collections::HashMap<RuneSource, Entity> =
        std::collections::HashMap::new();
    for (entity, view) in &existing {
        if let Some(duplicate) = by_source.insert(view.source, entity) {
            commands.entity(duplicate).despawn();
        }
    }

    for (idx, slot) in shop.stubs.iter().enumerate() {
        if let Some(stub) = slot {
            let src = RuneSource::Shop(idx);
            if by_source.remove(&src).is_none() {
                spawn_rune_entity(&mut commands, root_entity, *stub, src);
            }
        }
    }
    for (coord, stub) in grid.cells.iter() {
        let src = RuneSource::Grid(*coord);
        if by_source.remove(&src).is_none() {
            spawn_rune_entity(&mut commands, root_entity, *stub, src);
        }
    }
    for (idx, slot) in jokers.stubs.iter().enumerate() {
        if let Some(stub) = slot {
            let src = RuneSource::Joker(idx);
            if by_source.remove(&src).is_none() {
                spawn_rune_entity(&mut commands, root_entity, *stub, src);
            }
        }
    }

    for (_, entity) in by_source {
        commands.entity(entity).despawn();
    }
}

fn cursor_ui_pos(window: &Window, ui_scale: f32) -> Option<Vec2> {
    let scale = if ui_scale > 0.0 { ui_scale } else { 1.0 };
    window.cursor_position().map(|c| c / scale)
}

pub fn start_drag(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut shop: ResMut<ShopOffer>,
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    runes: Query<(Entity, &RuneView), Without<Dragging>>,
    dragging: Query<(), With<Dragging>>,
) {
    if !dragging.is_empty() {
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
        if home.distance(cursor) > radius {
            continue;
        }
        let Some(stub) = take_stub(view.source, &mut shop, &mut grid, &mut jokers) else {
            return;
        };
        commands.entity(entity).insert(Dragging {
            stub,
            from: view.source,
            grab_offset: cursor - home,
        });
        return;
    }
}

pub fn follow_cursor(
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut runes: Query<(&RuneView, Option<&Dragging>, &mut Node)>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let cursor = cursor_ui_pos(window, ui_scale.0);
    for (view, dragging, mut node) in &mut runes {
        let pos = match (dragging, cursor) {
            (Some(drag), Some(c)) => c - drag.grab_offset,
            _ => home_position(view.source),
        };
        node.left = Val::Px(pos.x - RUNE_DIAMETER * 0.5);
        node.top = Val::Px(pos.y - RUNE_DIAMETER * 0.5);
    }
}

fn find_drop_target(
    cursor: Vec2,
    kind: StubKind,
    grid: &RuneGrid,
    cells: &Query<&GridCellView>,
    shop_slots: &Query<&ShopSlotView>,
    joker_slots: &Query<&JokerSlotView>,
) -> Option<RuneSource> {
    let cell_r = CELL_DIAMETER * 0.5;
    let joker_r = JOKER_DIAMETER * 0.5;
    if kind == StubKind::Rune {
        for cell in cells {
            if !grid.is_unlocked(cell.coord) {
                continue;
            }
            if grid_cell_center(cell.coord).distance(cursor) <= cell_r {
                return Some(RuneSource::Grid(cell.coord));
            }
        }
    }
    if kind == StubKind::Joker {
        for slot in joker_slots {
            if joker_slot_center(slot.index).distance(cursor) <= joker_r {
                return Some(RuneSource::Joker(slot.index));
            }
        }
    }
    for slot in shop_slots {
        if shop_slot_center(slot.index).distance(cursor) <= cell_r {
            return Some(RuneSource::Shop(slot.index));
        }
    }
    None
}

fn take_stub(
    src: RuneSource,
    shop: &mut ShopOffer,
    grid: &mut RuneGrid,
    jokers: &mut JokerSlots,
) -> Option<RuneStub> {
    match src {
        RuneSource::Shop(idx) => shop.stubs[idx].take(),
        RuneSource::Grid(c) => grid.cells.remove(&c),
        RuneSource::Joker(idx) => jokers.stubs[idx].take(),
    }
}

fn place_stub(
    src: RuneSource,
    stub: RuneStub,
    shop: &mut ShopOffer,
    grid: &mut RuneGrid,
    jokers: &mut JokerSlots,
) {
    match src {
        RuneSource::Shop(idx) => shop.stubs[idx] = Some(stub),
        RuneSource::Grid(c) => {
            grid.cells.insert(c, stub);
        }
        RuneSource::Joker(idx) => jokers.stubs[idx] = Some(stub),
    }
}

fn peek_stub<'a>(
    src: RuneSource,
    shop: &'a ShopOffer,
    grid: &'a RuneGrid,
    jokers: &'a JokerSlots,
) -> Option<&'a RuneStub> {
    match src {
        RuneSource::Shop(idx) => shop.stubs[idx].as_ref(),
        RuneSource::Grid(c) => grid.cells.get(&c),
        RuneSource::Joker(idx) => jokers.stubs[idx].as_ref(),
    }
}

pub fn finish_drag(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut shop: ResMut<ShopOffer>,
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    mut views: Query<(Entity, &mut RuneView, Option<&Dragging>)>,
    cells: Query<&GridCellView>,
    shop_slots: Query<&ShopSlotView>,
    joker_slots: Query<&JokerSlotView>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }
    let Some((dragged_entity, drag)) = views
        .iter()
        .find_map(|(e, _, d)| d.map(|dd| (e, *dd)))
    else {
        return;
    };
    let Ok(window) = window.single() else {
        return;
    };
    let cursor = cursor_ui_pos(window, ui_scale.0);
    let target = cursor.map(|c| c - drag.grab_offset).and_then(|rune_center| {
        find_drop_target(
            rune_center,
            drag.stub.kind,
            &grid,
            &cells,
            &shop_slots,
            &joker_slots,
        )
    });

    let swap_ok = match target {
        Some(t) if t != drag.from => peek_stub(t, &shop, &grid, &jokers)
            .map(|d| can_place(d.kind, drag.from, &grid))
            .unwrap_or(true),
        _ => true,
    };

    match target {
        Some(t) if t != drag.from && swap_ok => {
            let displaced = take_stub(t, &mut shop, &mut grid, &mut jokers);
            place_stub(t, drag.stub, &mut shop, &mut grid, &mut jokers);
            if let Some(stub) = displaced {
                place_stub(drag.from, stub, &mut shop, &mut grid, &mut jokers);
                let other = views
                    .iter()
                    .find(|(e, v, d)| *e != dragged_entity && v.source == t && d.is_none())
                    .map(|(e, _, _)| e);
                if let Some(other_entity) = other {
                    if let Ok((_, mut v, _)) = views.get_mut(other_entity) {
                        v.source = drag.from;
                    }
                }
            }
            if let Ok((_, mut v, _)) = views.get_mut(dragged_entity) {
                v.source = t;
            }
        }
        _ => {
            place_stub(drag.from, drag.stub, &mut shop, &mut grid, &mut jokers);
        }
    }

    commands.entity(dragged_entity).remove::<Dragging>();
}

pub fn restore_dragged_on_exit(
    mut commands: Commands,
    mut shop: ResMut<ShopOffer>,
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    dragged: Query<(Entity, &Dragging)>,
) {
    for (entity, drag) in &dragged {
        place_stub(drag.from, drag.stub, &mut shop, &mut grid, &mut jokers);
        commands.entity(entity).remove::<Dragging>();
    }
}
