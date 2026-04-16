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

use super::{panel_radius, Viewport};

const RUNE_BORDER_WIDTH: f32 = 2.0;
const JOKER_BORDER_WIDTH: f32 = 4.0;

const SQRT_3: f32 = 1.732_050_8;
const CELL_GAP: f32 = 5.0;

const CELL_SIZE: f32 = 110.0;
const CELL_SIDE: f32 = CELL_SIZE / SQRT_3;
const CELL_DIAMETER: f32 = CELL_SIZE - CELL_GAP;
const RUNE_DIAMETER: f32 = 100.0;

const SHOP_MARGIN: f32 = 30.0;
const SHOP_SLOT_GAP: f32 = 30.0;
const SHOP_PANEL_RIGHT_MARGIN: f32 = 40.0;

const SHOP_PANEL_W: f32 = CELL_DIAMETER + 2.0 * SHOP_MARGIN;
const SHOP_PANEL_H: f32 = (SHOP_SLOTS as f32) * CELL_DIAMETER
    + ((SHOP_SLOTS - 1) as f32) * SHOP_SLOT_GAP
    + 2.0 * SHOP_MARGIN;

const START_RUN_BTN_W: f32 = 170.0;
const START_RUN_BTN_H: f32 = 60.0;
const START_RUN_BTN_RIGHT_MARGIN: f32 = 40.0;
const START_RUN_BTN_TOP_MARGIN: f32 = 24.0;

const JOKER_SLOT_DIAMETER: f32 = CELL_DIAMETER;
const JOKER_HEX_COORDS: [(i32, i32); JOKER_SLOTS] = [
    (4, -2),
    (2, -4),
    (-2, -2),
    (-4, 2),
    (-2, 4),
    (2, 2),
];

fn grid_center(viewport: &Viewport) -> Vec2 {
    Vec2::new(viewport.width * 0.5, viewport.height * 0.5)
}

fn shop_panel_pos(viewport: &Viewport) -> Vec2 {
    Vec2::new(
        viewport.width - SHOP_PANEL_W - SHOP_PANEL_RIGHT_MARGIN,
        (viewport.height - SHOP_PANEL_H) * 0.5,
    )
}

fn start_run_btn_pos(viewport: &Viewport) -> Vec2 {
    Vec2::new(
        viewport.width - START_RUN_BTN_W - START_RUN_BTN_RIGHT_MARGIN,
        START_RUN_BTN_TOP_MARGIN,
    )
}

fn grid_cell_center(viewport: &Viewport, coord: HexCoord) -> Vec2 {
    grid_center(viewport) + coord.to_pixel(CELL_SIDE)
}

fn shop_slot_center(viewport: &Viewport, idx: usize) -> Vec2 {
    let panel = shop_panel_pos(viewport);
    let first_y = panel.y + SHOP_MARGIN + CELL_DIAMETER * 0.5;
    let step = CELL_DIAMETER + SHOP_SLOT_GAP;
    Vec2::new(panel.x + SHOP_PANEL_W * 0.5, first_y + idx as f32 * step)
}

fn joker_slot_center(viewport: &Viewport, idx: usize) -> Vec2 {
    let (q, r) = JOKER_HEX_COORDS[idx];
    grid_center(viewport) + HexCoord::new(q, r).to_pixel(CELL_SIDE)
}

fn home_position(viewport: &Viewport, source: RuneSource) -> Vec2 {
    match source {
        RuneSource::Shop(idx) => shop_slot_center(viewport, idx),
        RuneSource::Grid(coord) => grid_cell_center(viewport, coord),
        RuneSource::Joker(idx) => joker_slot_center(viewport, idx),
    }
}

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct ShopRoot;

#[derive(Component)]
pub struct ShopPanel;

pub fn spawn_shop_screen(
    mut commands: Commands,
    run_state: Res<RunState>,
    money: Res<PlayerMoney>,
    viewport: Res<Viewport>,
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

    let btn_pos = start_run_btn_pos(&viewport);
    commands.spawn((
        ChildOf(root),
        Button,
        StartRunButton,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(btn_pos.x),
            top: Val::Px(btn_pos.y),
            width: Val::Px(START_RUN_BTN_W),
            height: Val::Px(START_RUN_BTN_H),
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

    let panel_pos = shop_panel_pos(&viewport);
    commands.spawn((
        ChildOf(root),
        ShopPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(panel_pos.x),
            top: Val::Px(panel_pos.y),
            width: Val::Px(SHOP_PANEL_W),
            height: Val::Px(SHOP_PANEL_H),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: panel_radius(),
            ..default()
        },
        BackgroundColor(palette::color("ui_panel_bg")),
        BorderColor::all(palette::color("ui_shop_slot_edge")),
    ));

    for coord in HexCoord::all_within_radius(GRID_RADIUS) {
        let center = grid_cell_center(&viewport, coord);
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
        let center = joker_slot_center(&viewport, idx);
        commands.spawn((
            ChildOf(root),
            JokerSlotView { index: idx },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - JOKER_SLOT_DIAMETER * 0.5),
                top: Val::Px(center.y - JOKER_SLOT_DIAMETER * 0.5),
                width: Val::Px(JOKER_SLOT_DIAMETER),
                height: Val::Px(JOKER_SLOT_DIAMETER),
                border: UiRect::all(Val::Px(3.0)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(palette::color("ui_joker_slot_bg")),
            BorderColor::all(palette::color("ui_joker_slot_edge")),
        ));
    }

    for idx in 0..SHOP_SLOTS {
        let center = shop_slot_center(&viewport, idx);
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
    viewport: &Viewport,
    stub: RuneStub,
    source: RuneSource,
) {
    let center = home_position(viewport, source);
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
    viewport: Res<Viewport>,
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
                spawn_rune_entity(&mut commands, root_entity, &viewport, *stub, src);
            }
        }
    }
    for (coord, stub) in grid.cells.iter() {
        let src = RuneSource::Grid(*coord);
        if by_source.remove(&src).is_none() {
            spawn_rune_entity(&mut commands, root_entity, &viewport, *stub, src);
        }
    }
    for (idx, slot) in jokers.stubs.iter().enumerate() {
        if let Some(stub) = slot {
            let src = RuneSource::Joker(idx);
            if by_source.remove(&src).is_none() {
                spawn_rune_entity(&mut commands, root_entity, &viewport, *stub, src);
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
    viewport: Res<Viewport>,
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
        let home = home_position(&viewport, view.source);
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
    viewport: Res<Viewport>,
    mut runes: Query<(&RuneView, Option<&Dragging>, &mut Node)>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let cursor = cursor_ui_pos(window, ui_scale.0);
    for (view, dragging, mut node) in &mut runes {
        let pos = match (dragging, cursor) {
            (Some(drag), Some(c)) => c - drag.grab_offset,
            _ => home_position(&viewport, view.source),
        };
        node.left = Val::Px(pos.x - RUNE_DIAMETER * 0.5);
        node.top = Val::Px(pos.y - RUNE_DIAMETER * 0.5);
    }
}

fn find_drop_target(
    cursor: Vec2,
    kind: StubKind,
    viewport: &Viewport,
    grid: &RuneGrid,
    cells: &Query<&GridCellView>,
    shop_slots: &Query<&ShopSlotView>,
    joker_slots: &Query<&JokerSlotView>,
) -> Option<RuneSource> {
    let cell_r = CELL_DIAMETER * 0.5;
    let joker_r = JOKER_SLOT_DIAMETER * 0.5;
    if kind == StubKind::Rune {
        for cell in cells {
            if !grid.is_unlocked(cell.coord) {
                continue;
            }
            if grid_cell_center(viewport, cell.coord).distance(cursor) <= cell_r {
                return Some(RuneSource::Grid(cell.coord));
            }
        }
    }
    if kind == StubKind::Joker {
        for slot in joker_slots {
            if joker_slot_center(viewport, slot.index).distance(cursor) <= joker_r {
                return Some(RuneSource::Joker(slot.index));
            }
        }
    }
    for slot in shop_slots {
        if shop_slot_center(viewport, slot.index).distance(cursor) <= cell_r {
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
    viewport: Res<Viewport>,
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
            &viewport,
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

pub fn reposition_shop_ui(
    viewport: Res<Viewport>,
    mut sets: ParamSet<(
        Query<(&GridCellView, &mut Node)>,
        Query<(&ShopSlotView, &mut Node)>,
        Query<(&JokerSlotView, &mut Node)>,
        Query<&mut Node, With<ShopPanel>>,
        Query<&mut Node, With<StartRunButton>>,
    )>,
) {
    if !viewport.is_changed() {
        return;
    }
    for (cell, mut node) in &mut sets.p0() {
        let center = grid_cell_center(&viewport, cell.coord);
        node.left = Val::Px(center.x - CELL_DIAMETER * 0.5);
        node.top = Val::Px(center.y - CELL_DIAMETER * 0.5);
    }
    for (slot, mut node) in &mut sets.p1() {
        let center = shop_slot_center(&viewport, slot.index);
        node.left = Val::Px(center.x - CELL_DIAMETER * 0.5);
        node.top = Val::Px(center.y - CELL_DIAMETER * 0.5);
    }
    for (slot, mut node) in &mut sets.p2() {
        let center = joker_slot_center(&viewport, slot.index);
        node.left = Val::Px(center.x - JOKER_SLOT_DIAMETER * 0.5);
        node.top = Val::Px(center.y - JOKER_SLOT_DIAMETER * 0.5);
    }
    for mut node in &mut sets.p3() {
        let pos = shop_panel_pos(&viewport);
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
    for mut node in &mut sets.p4() {
        let pos = start_run_btn_pos(&viewport);
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
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
