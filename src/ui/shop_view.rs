use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::PrimaryWindow;

use crate::palette;
use crate::run::PlayerMoney;
use crate::run::RunState;
use crate::rune::{
    can_place, write_pattern_contains, write_targets, Dragging, GridCellView, GridHighlights,
    HexCoord, IconAssets, JokerSlotView, JokerSlots, Rune, RuneGrid, RuneSource, RuneView,
    ShopOffer, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
use crate::transition::{Transition, TransitionAction};
use crate::wave::WavePhase;

use super::{panel_radius, Viewport};

const RUNE_BORDER_WIDTH: f32 = 2.0;
const JOKER_BORDER_WIDTH: f32 = 4.0;
const RUNE_ICON_INSET: f32 = 14.0;
const RUNE_HIGHLIGHT_ALPHA: f32 = 0.45;

const SQRT_3: f32 = 1.732_050_8;
const CELL_GAP: f32 = 5.0;

const CELL_SIZE: f32 = 110.0;
const CELL_SIDE: f32 = CELL_SIZE / SQRT_3;
const CELL_DIAMETER: f32 = CELL_SIZE - CELL_GAP;
const RUNE_DIAMETER: f32 = 100.0;

const SHOP_SLOT_GAP: f32 = 30.0;
const SHOP_RIGHT_MARGIN: f32 = 70.0;
const SHOP_COLUMN_W: f32 = CELL_DIAMETER;

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

const PRICE_LABEL_OFFSET_Y: f32 = 10.0;
const PRICE_LABEL_W: f32 = 60.0;
const PRICE_LABEL_H: f32 = 22.0;

const SHAKE_DURATION: f32 = 0.35;
const SHAKE_AMPLITUDE: f32 = 10.0;
const SHAKE_FREQ: f32 = 40.0;

fn grid_center(viewport: &Viewport) -> Vec2 {
    Vec2::new(viewport.width * 0.5, viewport.height * 0.5)
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
    let column_x = viewport.width - SHOP_RIGHT_MARGIN - SHOP_COLUMN_W * 0.5;
    let total_h =
        (SHOP_SLOTS as f32) * CELL_DIAMETER + ((SHOP_SLOTS - 1) as f32) * SHOP_SLOT_GAP;
    let first_y = (viewport.height - total_h) * 0.5 + CELL_DIAMETER * 0.5;
    let step = CELL_DIAMETER + SHOP_SLOT_GAP;
    Vec2::new(column_x, first_y + idx as f32 * step)
}

fn shop_price_label_pos(viewport: &Viewport, idx: usize) -> Vec2 {
    let slot = shop_slot_center(viewport, idx);
    Vec2::new(
        slot.x - PRICE_LABEL_W * 0.5,
        slot.y + CELL_DIAMETER * 0.5 + PRICE_LABEL_OFFSET_Y,
    )
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
pub struct HighlightOverlay;

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct ShopRoot;

#[derive(Component)]
pub struct ShopPriceLabel {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopShake {
    pub elapsed: f32,
}

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
        let pos = shop_price_label_pos(&viewport, idx);
        commands.spawn((
            ChildOf(root),
            ShopPriceLabel { index: idx },
            Text::new(""),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(palette::color("ui_text_money")),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x),
                top: Val::Px(pos.y),
                width: Val::Px(PRICE_LABEL_W),
                height: Val::Px(PRICE_LABEL_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(55),
        ));
    }
}

fn spawn_rune_entity(
    commands: &mut Commands,
    root: Entity,
    viewport: &Viewport,
    rune: Rune,
    source: RuneSource,
    icons: &IconAssets,
) {
    let center = home_position(viewport, source);
    let (border_width, border_color) = if rune.is_joker() {
        (JOKER_BORDER_WIDTH, palette::color("ui_joker_border"))
    } else {
        (RUNE_BORDER_WIDTH, palette::color("ui_rune_border"))
    };
    let icon_handle = rune
        .kind
        .and_then(|k| icons.for_stat(k.def().base_effect.0).cloned());
    let rune_entity = commands
        .spawn((
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
            BackgroundColor(rune.color),
            BorderColor::all(border_color),
        ))
        .id();
    if let Some(handle) = icon_handle {
        commands.spawn((
            ChildOf(rune_entity),
            ImageNode::new(handle).with_color(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(RUNE_ICON_INSET),
                top: Val::Px(RUNE_ICON_INSET),
                width: Val::Px(RUNE_DIAMETER - RUNE_ICON_INSET * 2.0),
                height: Val::Px(RUNE_DIAMETER - RUNE_ICON_INSET * 2.0),
                ..default()
            },
        ));
    }
    let inner = RUNE_DIAMETER - border_width * 2.0;
    commands.spawn((
        ChildOf(rune_entity),
        HighlightOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(inner),
            height: Val::Px(inner),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::NONE),
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

pub fn update_shop_price_labels(
    offer: Res<ShopOffer>,
    mut labels: Query<(&ShopPriceLabel, &mut Text, &mut Visibility)>,
) {
    for (label, mut text, mut visibility) in &mut labels {
        match offer.stubs.get(label.index).and_then(|s| s.as_ref()) {
            Some(rune) => {
                text.0 = format!("{}", rune.cost);
                *visibility = Visibility::Inherited;
            }
            None => {
                *visibility = Visibility::Hidden;
            }
        }
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
    icons: Res<IconAssets>,
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
        if let Some(rune) = slot {
            let src = RuneSource::Shop(idx);
            if by_source.remove(&src).is_none() {
                spawn_rune_entity(&mut commands, root_entity, &viewport, *rune, src, &icons);
            }
        }
    }
    for (coord, rune) in grid.cells.iter() {
        let src = RuneSource::Grid(*coord);
        if by_source.remove(&src).is_none() {
            spawn_rune_entity(&mut commands, root_entity, &viewport, *rune, src, &icons);
        }
    }
    for (idx, slot) in jokers.stubs.iter().enumerate() {
        if let Some(rune) = slot {
            let src = RuneSource::Joker(idx);
            if by_source.remove(&src).is_none() {
                spawn_rune_entity(&mut commands, root_entity, &viewport, *rune, src, &icons);
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
    money: Res<PlayerMoney>,
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
        if let RuneSource::Shop(_) = view.source {
            let cost = peek_rune(view.source, &shop, &grid, &jokers)
                .map(|r| r.cost)
                .unwrap_or(0);
            if !money.can_afford(cost) {
                commands.entity(entity).insert(ShopShake { elapsed: 0.0 });
                return;
            }
        }
        let Some(rune) = take_rune(view.source, &mut shop, &mut grid, &mut jokers) else {
            return;
        };
        commands.entity(entity).insert(Dragging {
            rune,
            from: view.source,
            grab_offset: cursor - home,
        });
        return;
    }
}

pub fn follow_cursor(
    mut commands: Commands,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    viewport: Res<Viewport>,
    mut runes: Query<(
        Entity,
        &RuneView,
        Option<&Dragging>,
        Option<&mut ShopShake>,
        &mut Node,
    )>,
) {
    let Ok(window) = window.single() else {
        return;
    };
    let cursor = cursor_ui_pos(window, ui_scale.0);
    let dt = time.delta_secs();
    for (entity, view, dragging, shake, mut node) in &mut runes {
        let mut pos = match (dragging, cursor) {
            (Some(drag), Some(c)) => c - drag.grab_offset,
            _ => home_position(&viewport, view.source),
        };
        if let Some(mut sh) = shake {
            sh.elapsed += dt;
            if sh.elapsed >= SHAKE_DURATION {
                commands.entity(entity).remove::<ShopShake>();
            } else {
                let decay = 1.0 - (sh.elapsed / SHAKE_DURATION);
                pos.x += (sh.elapsed * SHAKE_FREQ).sin() * SHAKE_AMPLITUDE * decay;
            }
        }
        node.left = Val::Px(pos.x - RUNE_DIAMETER * 0.5);
        node.top = Val::Px(pos.y - RUNE_DIAMETER * 0.5);
    }
}

fn find_drop_target(
    cursor: Vec2,
    is_joker: bool,
    viewport: &Viewport,
    grid: &RuneGrid,
    cells: &Query<&GridCellView>,
    joker_slots: &Query<&JokerSlotView>,
) -> Option<RuneSource> {
    let cell_r = CELL_DIAMETER * 0.5;
    let joker_r = JOKER_SLOT_DIAMETER * 0.5;
    if !is_joker {
        for cell in cells {
            if !grid.is_unlocked(cell.coord) {
                continue;
            }
            if grid_cell_center(viewport, cell.coord).distance(cursor) <= cell_r {
                return Some(RuneSource::Grid(cell.coord));
            }
        }
    } else {
        for slot in joker_slots {
            if joker_slot_center(viewport, slot.index).distance(cursor) <= joker_r {
                return Some(RuneSource::Joker(slot.index));
            }
        }
    }
    None
}

fn take_rune(
    src: RuneSource,
    shop: &mut ShopOffer,
    grid: &mut RuneGrid,
    jokers: &mut JokerSlots,
) -> Option<Rune> {
    match src {
        RuneSource::Shop(idx) => shop.stubs[idx].take(),
        RuneSource::Grid(c) => grid.cells.remove(&c),
        RuneSource::Joker(idx) => jokers.stubs[idx].take(),
    }
}

fn place_rune(
    src: RuneSource,
    rune: Rune,
    shop: &mut ShopOffer,
    grid: &mut RuneGrid,
    jokers: &mut JokerSlots,
) {
    match src {
        RuneSource::Shop(idx) => shop.stubs[idx] = Some(rune),
        RuneSource::Grid(c) => {
            grid.cells.insert(c, rune);
        }
        RuneSource::Joker(idx) => jokers.stubs[idx] = Some(rune),
    }
}

fn peek_rune<'a>(
    src: RuneSource,
    shop: &'a ShopOffer,
    grid: &'a RuneGrid,
    jokers: &'a JokerSlots,
) -> Option<&'a Rune> {
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
    mut money: ResMut<PlayerMoney>,
    mut shop: ResMut<ShopOffer>,
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    mut views: Query<(Entity, &mut RuneView, Option<&Dragging>)>,
    cells: Query<&GridCellView>,
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
            drag.rune.is_joker(),
            &viewport,
            &grid,
            &cells,
            &joker_slots,
        )
    });

    let from_shop = matches!(drag.from, RuneSource::Shop(_));
    let placement_ok = match target {
        Some(t) if t != drag.from => {
            let displaced = peek_rune(t, &shop, &grid, &jokers);
            if from_shop && displaced.is_some() {
                false
            } else {
                displaced
                    .map(|d| can_place(d.is_joker(), drag.from, &grid))
                    .unwrap_or(true)
            }
        }
        _ => false,
    };

    match target {
        Some(t) if placement_ok => {
            if from_shop {
                money.spend(drag.rune.cost);
            }
            let displaced = take_rune(t, &mut shop, &mut grid, &mut jokers);
            place_rune(t, drag.rune, &mut shop, &mut grid, &mut jokers);
            if let Some(rune) = displaced {
                place_rune(drag.from, rune, &mut shop, &mut grid, &mut jokers);
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
            place_rune(drag.from, drag.rune, &mut shop, &mut grid, &mut jokers);
        }
    }

    commands.entity(dragged_entity).remove::<Dragging>();
}

pub fn reposition_shop_ui(
    viewport: Res<Viewport>,
    mut sets: ParamSet<(
        Query<(&GridCellView, &mut Node)>,
        Query<(&JokerSlotView, &mut Node)>,
        Query<&mut Node, With<StartRunButton>>,
        Query<(&ShopPriceLabel, &mut Node)>,
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
        let center = joker_slot_center(&viewport, slot.index);
        node.left = Val::Px(center.x - JOKER_SLOT_DIAMETER * 0.5);
        node.top = Val::Px(center.y - JOKER_SLOT_DIAMETER * 0.5);
    }
    for mut node in &mut sets.p2() {
        let pos = start_run_btn_pos(&viewport);
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
    for (label, mut node) in &mut sets.p3() {
        let pos = shop_price_label_pos(&viewport, label.index);
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
}

pub fn update_highlights(
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    viewport: Res<Viewport>,
    grid: Res<RuneGrid>,
    dragging: Query<&Dragging>,
    views: Query<&RuneView, Without<Dragging>>,
    cells: Query<&GridCellView>,
    joker_slots: Query<&JokerSlotView>,
    mut highlights: ResMut<GridHighlights>,
) {
    highlights.write_targets.clear();
    highlights.write_sources.clear();

    let Ok(window) = window_q.single() else { return };
    let Some(cursor) = cursor_ui_pos(window, ui_scale.0) else { return };

    if let Some(drag) = dragging.iter().next() {
        let Some(kind) = drag.rune.kind else { return };
        let Some(write) = kind.def().write else { return };
        let rune_center = cursor - drag.grab_offset;
        let Some(target) = find_drop_target(
            rune_center,
            drag.rune.is_joker(),
            &viewport,
            &grid,
            &cells,
            &joker_slots,
        ) else { return };
        let RuneSource::Grid(c) = target else { return };
        for t in write_targets(&write, c, &grid) {
            highlights.write_targets.insert(t);
        }
        return;
    }

    let radius = RUNE_DIAMETER * 0.5;
    let mut hovered: Option<HexCoord> = None;
    for view in &views {
        if let RuneSource::Grid(c) = view.source {
            if grid_cell_center(&viewport, c).distance(cursor) <= radius {
                hovered = Some(c);
                break;
            }
        }
    }
    let Some(coord) = hovered else { return };
    let Some(rune) = grid.cells.get(&coord) else { return };
    let Some(kind) = rune.kind else { return };

    if let Some(write) = kind.def().write {
        for t in write_targets(&write, coord, &grid) {
            highlights.write_targets.insert(t);
        }
    }

    for (src_coord, src_rune) in grid.cells.iter() {
        if *src_coord == coord { continue }
        let Some(src_kind) = src_rune.kind else { continue };
        let Some(write) = src_kind.def().write else { continue };
        if write_pattern_contains(&write, *src_coord, coord) {
            highlights.write_sources.insert(*src_coord);
        }
    }
}

pub fn apply_highlights(
    highlights: Res<GridHighlights>,
    runes: Query<&RuneView>,
    mut overlays: Query<(&ChildOf, &mut BackgroundColor), With<HighlightOverlay>>,
) {
    for (child_of, mut bg) in &mut overlays {
        let source = runes.get(child_of.0).map(|v| v.source).ok();
        let color = match source {
            Some(RuneSource::Grid(coord)) => {
                if highlights.write_targets.contains(&coord) {
                    palette::color("ui_rune_write_target").with_alpha(RUNE_HIGHLIGHT_ALPHA)
                } else if highlights.write_sources.contains(&coord) {
                    palette::color("ui_rune_write_source").with_alpha(RUNE_HIGHLIGHT_ALPHA)
                } else {
                    Color::NONE
                }
            }
            _ => Color::NONE,
        };
        *bg = BackgroundColor(color);
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
        place_rune(drag.from, drag.rune, &mut shop, &mut grid, &mut jokers);
        commands.entity(entity).remove::<Dragging>();
    }
}
