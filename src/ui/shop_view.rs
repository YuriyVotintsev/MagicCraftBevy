use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::PrimaryWindow;

use crate::balance::GameBalance;
use crate::palette;
use crate::run::PlayerMoney;
use crate::run::RunState;
use crate::rune::{
    can_place, incoming_factor, roll_shop_offer, write_pattern_contains, write_pattern_coords,
    write_targets, Dragging, GridCellView, GridHighlights, HexCoord, IconAssets, JokerSlotView,
    JokerSlots, Pattern, RerollState, Rune, RuneCosts, RuneGrid, RuneKind, RuneSource, RuneView,
    ShopOffer, Tier, Write, WriteEffect, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
use crate::stats::StatDisplayRegistry;
use crate::transition::{Transition, TransitionAction};
use crate::wave::WavePhase;

use super::stat_line_builder::{StatLineBuilder, StatRenderMode};
use super::widgets::{button_node, panel_node, ReleasedButtons};
use super::Viewport;

const RUNE_ICON_INSET: f32 = 14.0;

const RING_THICKNESS: f32 = 7.0;
const RING_OUTER_INSET: f32 = 20.0;
const RING_INNER_INSET: f32 = 30.0;

const RUNE_EDGE_RADIUS: f32 = RUNE_DIAMETER * 0.5;
const TARGET_RING_RADIUS: f32 = (RUNE_DIAMETER - RING_OUTER_INSET * 2.0) * 0.5;
const SOURCE_RING_RADIUS: f32 = (RUNE_DIAMETER - RING_INNER_INSET * 2.0) * 0.5;

const ARROW_LINE_THICKNESS: f32 = 5.0;
const ARROW_HEAD_LENGTH: f32 = 15.0;
const ARROW_HEAD_THICKNESS: f32 = 5.0;
const ARROW_HEAD_ANGLE_DEG: f32 = 35.0;
const ARROW_PARALLEL_OFFSET: f32 = 7.0;

const CELL_PATTERN_BORDER: f32 = 6.0;

const TOOLTIP_W: f32 = 320.0;
const TOOLTIP_PAD: f32 = 14.0;
const TOOLTIP_RIGHT: f32 = 40.0;
const TOOLTIP_BOTTOM: f32 = 40.0;

const SHADOW_OFFSET_X: f32 = 3.0;
const SHADOW_OFFSET_Y: f32 = 3.0;
const SHADOW_ALPHA: f32 = 0.5;

fn shadow_color() -> Color {
    Color::BLACK.with_alpha(SHADOW_ALPHA)
}

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

const REROLL_BTN_W: f32 = SHOP_COLUMN_W;
const REROLL_BTN_H: f32 = 50.0;
const REROLL_BTN_GAP: f32 = 20.0;

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

fn reroll_btn_pos(viewport: &Viewport) -> Vec2 {
    let last = shop_slot_center(viewport, SHOP_SLOTS - 1);
    Vec2::new(
        last.x - REROLL_BTN_W * 0.5,
        last.y + CELL_DIAMETER * 0.5 + REROLL_BTN_GAP,
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

#[derive(Component, Copy, Clone)]
pub enum HighlightRing {
    Target,
    Source,
}

#[derive(Component)]
pub struct HighlightArrow;

#[derive(Component)]
pub struct RuneTooltipRoot;

#[derive(Component)]
pub struct HighlightShadow;

#[derive(Component)]
pub struct CellPatternShadow {
    pub coord: HexCoord,
}

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Component)]
pub struct RerollButton;

#[derive(Component)]
pub struct RerollButtonLabel;

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
        Text(format!("Wave {}", run_state.wave)),
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
        StartRunButton,
        button_node(
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(btn_pos.x),
                top: Val::Px(btn_pos.y),
                width: Val::Px(START_RUN_BTN_W),
                height: Val::Px(START_RUN_BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        ),
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
            CellPatternShadow { coord },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - CELL_DIAMETER * 0.5 + SHADOW_OFFSET_X),
                top: Val::Px(center.y - CELL_DIAMETER * 0.5 + SHADOW_OFFSET_Y),
                width: Val::Px(CELL_DIAMETER),
                height: Val::Px(CELL_DIAMETER),
                border: UiRect::all(Val::Px(CELL_PATTERN_BORDER)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
        ));
        commands.spawn((
            ChildOf(root),
            GridCellView { coord },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - CELL_DIAMETER * 0.5),
                top: Val::Px(center.y - CELL_DIAMETER * 0.5),
                width: Val::Px(CELL_DIAMETER),
                height: Val::Px(CELL_DIAMETER),
                border: UiRect::all(Val::Px(CELL_PATTERN_BORDER)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(palette::color("ui_cell_locked_bg")),
            BorderColor::all(Color::NONE),
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

    let reroll_pos = reroll_btn_pos(&viewport);
    commands.spawn((
        ChildOf(root),
        RerollButton,
        button_node(
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(reroll_pos.x),
                top: Val::Px(reroll_pos.y),
                width: Val::Px(REROLL_BTN_W),
                height: Val::Px(REROLL_BTN_H),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        ),
        children![(
            RerollButtonLabel,
            Text::new(""),
            TextFont { font_size: 18.0, ..default() },
            TextColor(palette::color("ui_text")),
        )],
    ));

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
    let icon_handle = rune
        .kind
        .and_then(|k| icons.for_stat(k.def().base_effect.0).cloned());
    let rune_entity = commands
        .spawn((
            ChildOf(root),
            RuneView { source, rune_id: rune.id },
            GlobalZIndex(60),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(center.x - RUNE_DIAMETER * 0.5),
                top: Val::Px(center.y - RUNE_DIAMETER * 0.5),
                width: Val::Px(RUNE_DIAMETER),
                height: Val::Px(RUNE_DIAMETER),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(rune.color),
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
    spawn_ring_pair(commands, rune_entity, HighlightRing::Target, RING_OUTER_INSET);
    spawn_ring_pair(commands, rune_entity, HighlightRing::Source, RING_INNER_INSET);
}

fn spawn_ring_pair(commands: &mut Commands, parent: Entity, kind: HighlightRing, inset: f32) {
    let diameter = RUNE_DIAMETER - inset * 2.0;
    let mut spawn_one = |left: f32, top: f32, shadow: bool| {
        let mut e = commands.spawn((
            ChildOf(parent),
            kind,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                width: Val::Px(diameter),
                height: Val::Px(diameter),
                border: UiRect::all(Val::Px(RING_THICKNESS)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
        ));
        if shadow {
            e.insert(HighlightShadow);
        }
    };
    spawn_one(inset + SHADOW_OFFSET_X, inset + SHADOW_OFFSET_Y, true);
    spawn_one(inset, inset, false);
}

pub fn reset_reroll_cost(balance: Res<GameBalance>, mut reroll: ResMut<RerollState>) {
    reroll.cost = balance.runes.reroll_base_cost;
}

pub fn reroll_button_system(
    buttons: ReleasedButtons<RerollButton>,
    mut money: ResMut<PlayerMoney>,
    mut offer: ResMut<ShopOffer>,
    grid: Res<RuneGrid>,
    balance: Res<GameBalance>,
    costs: Res<RuneCosts>,
    mut reroll: ResMut<RerollState>,
    dragging: Query<(), With<Dragging>>,
) {
    buttons.for_each(|_| {
        if !dragging.is_empty() { return }
        if !money.can_afford(reroll.cost) { return }
        money.spend(reroll.cost);
        roll_shop_offer(&mut offer, &grid, &balance, &costs);
        reroll.cost = reroll.cost.saturating_add(balance.runes.reroll_cost_step);
    });
}

pub fn update_reroll_label(
    reroll: Res<RerollState>,
    money: Res<PlayerMoney>,
    buttons: Query<&Children, With<RerollButton>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<RerollButtonLabel>>,
) {
    if !reroll.is_changed() && !money.is_changed() { return }
    let affordable = money.can_afford(reroll.cost);
    for children in &buttons {
        for c in children.iter() {
            let Ok((mut text, mut color)) = texts.get_mut(c) else { continue };
            text.0 = format!("Reroll ({})", reroll.cost);
            *color = TextColor(if affordable {
                palette::color("ui_text")
            } else {
                palette::color("ui_text_disabled")
            });
        }
    }
}

pub fn start_run_system(
    buttons: ReleasedButtons<StartRunButton>,
    mut transition: ResMut<Transition>,
) {
    buttons.for_each(|_| {
        transition.request(TransitionAction::Wave(WavePhase::Combat));
    });
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
    mut cells: Query<(&GridCellView, &mut BackgroundColor)>,
) {
    for (cell, mut bg) in &mut cells {
        let bg_alias = if grid.is_unlocked(cell.coord) {
            "ui_cell_unlocked_bg"
        } else {
            "ui_cell_locked_bg"
        };
        *bg = BackgroundColor(palette::color(bg_alias));
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

    let mut by_source: std::collections::HashMap<RuneSource, (Entity, u32)> =
        std::collections::HashMap::new();
    for (entity, view) in &existing {
        if let Some((duplicate, _)) = by_source.insert(view.source, (entity, view.rune_id)) {
            commands.entity(duplicate).despawn();
        }
    }

    let mut reconcile_slot = |commands: &mut Commands, src: RuneSource, rune: Rune| {
        match by_source.remove(&src) {
            Some((_, id)) if id == rune.id => {}
            Some((entity, _)) => {
                commands.entity(entity).despawn();
                spawn_rune_entity(commands, root_entity, &viewport, rune, src, &icons);
            }
            None => {
                spawn_rune_entity(commands, root_entity, &viewport, rune, src, &icons);
            }
        }
    };

    for (idx, slot) in shop.stubs.iter().enumerate() {
        if let Some(rune) = slot {
            reconcile_slot(&mut commands, RuneSource::Shop(idx), *rune);
        }
    }
    for (coord, rune) in grid.cells.iter() {
        reconcile_slot(&mut commands, RuneSource::Grid(*coord), *rune);
    }
    for (idx, slot) in jokers.stubs.iter().enumerate() {
        if let Some(rune) = slot {
            reconcile_slot(&mut commands, RuneSource::Joker(idx), *rune);
        }
    }

    for (_, (entity, _)) in by_source {
        commands.entity(entity).despawn();
    }
}

pub(super) fn cursor_ui_pos(window: &Window, ui_scale: f32) -> Option<Vec2> {
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

pub(super) fn find_drop_target(
    cursor: Vec2,
    is_joker: bool,
    viewport: &Viewport,
    grid: &RuneGrid,
    _cells: &Query<&GridCellView>,
    joker_slots: &Query<&JokerSlotView>,
) -> Option<RuneSource> {
    let joker_r = JOKER_SLOT_DIAMETER * 0.5;
    if !is_joker {
        let coord = HexCoord::from_pixel(cursor - grid_center(viewport), CELL_SIDE);
        if coord.ring_radius() > GRID_RADIUS { return None }
        if !grid.is_unlocked(coord) { return None }
        return Some(RuneSource::Grid(coord));
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
        Query<&mut Node, With<RerollButton>>,
        Query<(&CellPatternShadow, &mut Node)>,
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
    for mut node in &mut sets.p4() {
        let pos = reroll_btn_pos(&viewport);
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
    for (shadow, mut node) in &mut sets.p5() {
        let center = grid_cell_center(&viewport, shadow.coord);
        node.left = Val::Px(center.x - CELL_DIAMETER * 0.5 + SHADOW_OFFSET_X);
        node.top = Val::Px(center.y - CELL_DIAMETER * 0.5 + SHADOW_OFFSET_Y);
    }
}

pub fn update_highlights(
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    viewport: Res<Viewport>,
    grid: Res<RuneGrid>,
    dragging: Query<(Entity, &Dragging)>,
    views: Query<(Entity, &RuneView), Without<Dragging>>,
    cells: Query<&GridCellView>,
    joker_slots: Query<&JokerSlotView>,
    mut highlights: ResMut<GridHighlights>,
) {
    highlights.center_entity = None;
    highlights.center_pos = None;
    highlights.write_targets.clear();
    highlights.write_sources.clear();
    highlights.pattern_cells.clear();

    let Ok(window) = window_q.single() else { return };
    let Some(cursor) = cursor_ui_pos(window, ui_scale.0) else { return };

    if let Some((drag_entity, drag)) = dragging.iter().next() {
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
        highlights.center_entity = Some(drag_entity);
        highlights.center_pos = Some(rune_center);

        if let Some(write) = drag.rune.kind.and_then(|k| k.def().write) {
            for t in write_targets(&write, c, &grid) {
                highlights.write_targets.insert(t);
            }
            for p in write_pattern_coords(&write, c) {
                highlights.pattern_cells.insert(p);
            }
        }

        for (src_coord, src_rune) in grid.cells.iter() {
            if *src_coord == c { continue }
            let Some(src_kind) = src_rune.kind else { continue };
            let Some(write) = src_kind.def().write else { continue };
            if write_pattern_contains(&write, *src_coord, c) {
                highlights.write_sources.insert(*src_coord);
            }
        }
        return;
    }

    let hover_coord = {
        let c = HexCoord::from_pixel(cursor - grid_center(&viewport), CELL_SIDE);
        if c.ring_radius() <= GRID_RADIUS { Some(c) } else { None }
    };
    let hovered = hover_coord.and_then(|hc| {
        views.iter().find_map(|(e, v)| match v.source {
            RuneSource::Grid(c) if c == hc => Some((e, c)),
            _ => None,
        })
    });
    let Some((entity, coord)) = hovered else { return };
    let Some(rune) = grid.cells.get(&coord) else { return };
    let Some(kind) = rune.kind else { return };
    highlights.center_entity = Some(entity);
    highlights.center_pos = Some(grid_cell_center(&viewport, coord));

    if let Some(write) = kind.def().write {
        for t in write_targets(&write, coord, &grid) {
            highlights.write_targets.insert(t);
        }
        for p in write_pattern_coords(&write, coord) {
            highlights.pattern_cells.insert(p);
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
    mut commands: Commands,
    viewport: Res<Viewport>,
    highlights: Res<GridHighlights>,
    root: Query<Entity, With<ShopRoot>>,
    runes: Query<&RuneView>,
    mut rings: Query<(&ChildOf, &HighlightRing, Option<&HighlightShadow>, &mut BorderColor), (Without<GridCellView>, Without<CellPatternShadow>)>,
    mut cell_borders: Query<(&GridCellView, &mut BorderColor), (Without<HighlightRing>, Without<CellPatternShadow>)>,
    mut cell_shadows: Query<(&CellPatternShadow, &mut BorderColor), (Without<HighlightRing>, Without<GridCellView>)>,
    arrows: Query<Entity, With<HighlightArrow>>,
) {
    let pattern_color = palette::color("ui_rune_write_target");
    for (cell, mut border) in &mut cell_borders {
        let color = if highlights.pattern_cells.contains(&cell.coord) {
            pattern_color
        } else {
            Color::NONE
        };
        *border = BorderColor::all(color);
    }
    for (shadow, mut border) in &mut cell_shadows {
        let color = if highlights.pattern_cells.contains(&shadow.coord) {
            shadow_color()
        } else {
            Color::NONE
        };
        *border = BorderColor::all(color);
    }

    for (child_of, kind, shadow, mut border) in &mut rings {
        let coord = match runes.get(child_of.0).map(|v| v.source) {
            Ok(RuneSource::Grid(c)) => Some(c),
            _ => None,
        };
        let matches = match kind {
            HighlightRing::Target => {
                highlights.center_entity == Some(child_of.0)
                    && !highlights.write_targets.is_empty()
            }
            HighlightRing::Source => coord
                .map(|c| highlights.write_sources.contains(&c))
                .unwrap_or(false),
        };
        let color = if matches {
            if shadow.is_some() {
                shadow_color()
            } else {
                match kind {
                    HighlightRing::Target => palette::color("ui_rune_write_target"),
                    HighlightRing::Source => palette::color("ui_rune_write_source"),
                }
            }
        } else {
            Color::NONE
        };
        *border = BorderColor::all(color);
    }

    for entity in &arrows {
        commands.entity(entity).despawn();
    }

    let Ok(root_entity) = root.single() else { return };
    let Some(center_pos) = highlights.center_pos else { return };

    for target in &highlights.write_targets {
        let target_pos = grid_cell_center(&viewport, *target);
        let direction = (target_pos - center_pos).normalize_or_zero();
        if direction == Vec2::ZERO { continue }
        let offset = if highlights.write_sources.contains(target) {
            Vec2::new(-direction.y, direction.x) * ARROW_PARALLEL_OFFSET
        } else {
            Vec2::ZERO
        };
        let tail = center_pos + direction * TARGET_RING_RADIUS + offset;
        let head = target_pos - direction * RUNE_EDGE_RADIUS + offset;
        spawn_arrow(&mut commands, root_entity, tail, head,
            palette::color("ui_rune_write_target"));
    }

    for source in &highlights.write_sources {
        let source_pos = grid_cell_center(&viewport, *source);
        let direction = (center_pos - source_pos).normalize_or_zero();
        if direction == Vec2::ZERO { continue }
        let offset = if highlights.write_targets.contains(source) {
            Vec2::new(-direction.y, direction.x) * ARROW_PARALLEL_OFFSET
        } else {
            Vec2::ZERO
        };
        let tail = source_pos + direction * SOURCE_RING_RADIUS + offset;
        let head = center_pos - direction * RUNE_EDGE_RADIUS + offset;
        spawn_arrow(&mut commands, root_entity, tail, head,
            palette::color("ui_rune_write_source"));
    }
}

fn spawn_arrow(commands: &mut Commands, root: Entity, tail: Vec2, head: Vec2, color: Color) {
    let delta = head - tail;
    let length = delta.length();
    if length < 1.0 { return }

    let head_length = ARROW_HEAD_LENGTH.min(length * 0.5);
    let base_angle = delta.y.atan2(delta.x);
    let half = ARROW_HEAD_ANGLE_DEG.to_radians();
    let shadow_off = Vec2::new(SHADOW_OFFSET_X, SHADOW_OFFSET_Y);
    let sc = shadow_color();

    // shadows first (rendered below real)
    spawn_rotated_rect(commands, root, tail + shadow_off, head + shadow_off, ARROW_LINE_THICKNESS, sc, 64);
    for side_angle in [base_angle + std::f32::consts::PI - half, base_angle + std::f32::consts::PI + half] {
        let side_dir = Vec2::new(side_angle.cos(), side_angle.sin());
        let end = head + side_dir * head_length;
        spawn_rotated_rect(commands, root, head + shadow_off, end + shadow_off, ARROW_HEAD_THICKNESS, sc, 64);
    }

    spawn_rotated_rect(commands, root, tail, head, ARROW_LINE_THICKNESS, color, 65);
    for side_angle in [base_angle + std::f32::consts::PI - half, base_angle + std::f32::consts::PI + half] {
        let side_dir = Vec2::new(side_angle.cos(), side_angle.sin());
        let end = head + side_dir * head_length;
        spawn_rotated_rect(commands, root, head, end, ARROW_HEAD_THICKNESS, color, 65);
    }
}

fn spawn_rotated_rect(
    commands: &mut Commands,
    root: Entity,
    a: Vec2,
    b: Vec2,
    thickness: f32,
    color: Color,
    z: i32,
) {
    let delta = b - a;
    let length = delta.length();
    if length < 0.1 { return }
    let mid = (a + b) * 0.5;
    let angle = delta.y.atan2(delta.x);
    commands.spawn((
        ChildOf(root),
        HighlightArrow,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(mid.x - length * 0.5),
            top: Val::Px(mid.y - thickness * 0.5),
            width: Val::Px(length),
            height: Val::Px(thickness),
            ..default()
        },
        BackgroundColor(color),
        UiTransform::from_rotation(Rot2::radians(angle)),
        GlobalZIndex(z),
    ));
}

fn rune_name(kind: RuneKind) -> &'static str {
    match kind {
        RuneKind::Spike => "Spike",
        RuneKind::HeartStone => "HeartStone",
        RuneKind::Resonator => "Resonator",
    }
}

fn tier_label(tier: Tier) -> &'static str {
    match tier {
        Tier::Common => "Common",
        Tier::Rare => "Rare",
    }
}

fn limit_label(limit: Option<u32>) -> String {
    limit
        .map(|n| format!("Max {}", n))
        .unwrap_or_else(|| "\u{221e}".to_string())
}

fn write_label(write: &Write) -> String {
    let pattern = match write.pattern {
        Pattern::Adjacent => "Adjacent",
    };
    let effect = match write.effect {
        WriteEffect::More { factor } => format!("+{:.0}% base effect", (factor - 1.0) * 100.0),
    };
    format!("{}: {}", pattern, effect)
}

#[allow(clippy::too_many_arguments)]
pub fn update_tooltip(
    mut commands: Commands,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    viewport: Res<Viewport>,
    grid: Res<RuneGrid>,
    shop: Res<ShopOffer>,
    registry: Res<StatDisplayRegistry>,
    root: Query<Entity, With<ShopRoot>>,
    dragging: Query<&Dragging>,
    cells: Query<&GridCellView>,
    joker_slots: Query<&JokerSlotView>,
    existing: Query<Entity, With<RuneTooltipRoot>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Ok(window) = window_q.single() else { return };
    let Some(cursor) = cursor_ui_pos(window, ui_scale.0) else { return };
    let Ok(root_entity) = root.single() else { return };

    let mut result: Option<(Rune, f32)> = None;
    if let Some(drag) = dragging.iter().next() {
        let rune_center = cursor - drag.grab_offset;
        let target = find_drop_target(
            rune_center,
            drag.rune.is_joker(),
            &viewport,
            &grid,
            &cells,
            &joker_slots,
        );
        let factor = match target {
            Some(RuneSource::Grid(c)) => incoming_factor(c, &grid),
            _ => 1.0,
        };
        result = Some((drag.rune, factor));
    } else {
        for idx in 0..SHOP_SLOTS {
            let center = shop_slot_center(&viewport, idx);
            if center.distance(cursor) <= CELL_DIAMETER * 0.5 {
                if let Some(r) = shop.stubs[idx] {
                    result = Some((r, 1.0));
                    break;
                }
            }
        }
        if result.is_none() {
            let coord = HexCoord::from_pixel(cursor - grid_center(&viewport), CELL_SIDE);
            if coord.ring_radius() <= GRID_RADIUS {
                if let Some(r) = grid.cells.get(&coord).copied() {
                    let factor = incoming_factor(coord, &grid);
                    result = Some((r, factor));
                }
            }
        }
    }

    let Some((rune, factor)) = result else { return };
    let Some(kind) = rune.kind else { return };
    let def = kind.def();
    let (base_stat, base_kind, base_value) = def.base_effect;
    let effective_value = base_value * factor;

    let tooltip = commands
        .spawn((
            ChildOf(root_entity),
            RuneTooltipRoot,
            panel_node(
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(TOOLTIP_RIGHT),
                    bottom: Val::Px(TOOLTIP_BOTTOM),
                    width: Val::Px(TOOLTIP_W),
                    padding: UiRect::all(Val::Px(TOOLTIP_PAD)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                None,
            ),
            GlobalZIndex(70),
        ))
        .id();

    commands.spawn((
        ChildOf(tooltip),
        Text::new(rune_name(kind)),
        TextFont { font_size: 22.0, ..default() },
        TextColor(palette::color("ui_text")),
    ));
    commands.spawn((
        ChildOf(tooltip),
        Text::new(tier_label(def.tier)),
        TextFont { font_size: 14.0, ..default() },
        TextColor(palette::color("ui_text_subtle")),
    ));

    let formats = registry.get_format(&[(base_stat, base_kind)]);
    if let Some(spans) = formats.first() {
        let line = StatLineBuilder::spawn_line(
            &mut commands,
            spans,
            StatRenderMode::Effective {
                effective: &[effective_value],
                raw: &[base_value],
            },
            16.0,
        );
        commands.entity(line).insert(ChildOf(tooltip));
    }

    if let Some(write) = def.write {
        commands.spawn((
            ChildOf(tooltip),
            Text::new(write_label(&write)),
            TextFont { font_size: 14.0, ..default() },
            TextColor(palette::color("ui_text")),
        ));
    }

    commands.spawn((
        ChildOf(tooltip),
        Text::new(limit_label(def.limit)),
        TextFont { font_size: 14.0, ..default() },
        TextColor(palette::color("ui_text_subtle")),
    ));
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
