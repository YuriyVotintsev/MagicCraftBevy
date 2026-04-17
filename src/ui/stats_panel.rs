use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::PrimaryWindow;

use crate::actors::compute_player_stats;
use crate::palette;
use crate::rune::{Dragging, GridCellView, JokerSlotView, RuneGrid, RuneSource};
use crate::stats::{ComputedStats, Stat, StatCalculators, StatDisplayRegistry};

use super::shop_view::{cursor_ui_pos, find_drop_target, ShopRoot};
use super::stat_line_builder::{StatLineBuilder, StatRenderMode};
use super::{panel_radius, Viewport};

const PANEL_LEFT: f32 = 40.0;
const PANEL_WIDTH: f32 = 320.0;
const PANEL_PADDING: f32 = 14.0;
const ROW_FONT: f32 = 16.0;
const HEADER_FONT: f32 = 22.0;

#[derive(Resource, Default)]
pub struct StatsPanelState {
    pub current: ComputedStats,
    pub preview: Option<ComputedStats>,
}

#[derive(Component)]
pub struct StatsPanelAnchor;

#[derive(Component)]
pub struct StatsPanelRoot;

#[derive(Component)]
pub struct StatsPanelList;

#[derive(Component)]
pub struct StatRow;

pub fn spawn_stats_panel(
    mut commands: Commands,
    root: Query<Entity, With<ShopRoot>>,
) {
    let Ok(root_entity) = root.single() else {
        return;
    };

    let anchor = commands
        .spawn((
            ChildOf(root_entity),
            StatsPanelAnchor,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PANEL_LEFT),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            GlobalZIndex(55),
        ))
        .id();

    let panel = commands
        .spawn((
            ChildOf(anchor),
            StatsPanelRoot,
            Node {
                width: Val::Px(PANEL_WIDTH),
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                border_radius: panel_radius(),
                ..default()
            },
            BackgroundColor(palette::color("ui_panel_bg")),
        ))
        .id();

    commands.spawn((
        ChildOf(panel),
        Text::new("Stats"),
        TextFont {
            font_size: HEADER_FONT,
            ..default()
        },
        TextColor(palette::color("ui_text")),
    ));

    commands.spawn((
        ChildOf(panel),
        StatsPanelList,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn compute_state(
    mut state: ResMut<StatsPanelState>,
    grid: Res<RuneGrid>,
    calculators: Res<StatCalculators>,
    viewport: Res<Viewport>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    dragging: Query<&Dragging>,
    cells: Query<&GridCellView>,
    joker_slots: Query<&JokerSlotView>,
) {
    let drag = dragging.iter().next().copied();

    let mut committed = (*grid).clone();
    if let Some(d) = drag {
        if let RuneSource::Grid(c) = d.from {
            committed.cells.insert(c, d.rune);
        }
    }
    let (_, current) = compute_player_stats(&committed, &calculators);
    state.current = current;
    state.preview = None;

    let Some(d) = drag else { return };
    let Ok(window) = window_q.single() else { return };
    let Some(cursor) = cursor_ui_pos(window, ui_scale.0) else { return };
    let rune_center = cursor - d.grab_offset;
    let Some(target) = find_drop_target(
        rune_center,
        d.rune.is_joker(),
        &viewport,
        &grid,
        &cells,
        &joker_slots,
    ) else {
        return;
    };
    if target == d.from {
        return;
    }

    let mut preview_grid = committed.clone();
    let from_shop = matches!(d.from, RuneSource::Shop(_));

    if let RuneSource::Grid(fc) = d.from {
        preview_grid.cells.remove(&fc);
    }
    match target {
        RuneSource::Grid(tc) => {
            let displaced = preview_grid.cells.get(&tc).copied();
            if from_shop && displaced.is_some() {
                return;
            }
            preview_grid.cells.insert(tc, d.rune);
            if let (Some(b), RuneSource::Grid(fc)) = (displaced, d.from) {
                preview_grid.cells.insert(fc, b);
            }
        }
        RuneSource::Joker(_) => {
            // Jokers don't contribute to stats; drop removes grid rune if source was grid.
        }
        RuneSource::Shop(_) => return,
    }

    let (_, preview) = compute_player_stats(&preview_grid, &calculators);
    state.preview = Some(preview);
}

pub fn render(
    mut commands: Commands,
    state: Res<StatsPanelState>,
    registry: Res<StatDisplayRegistry>,
    list: Query<Entity, With<StatsPanelList>>,
    existing: Query<Entity, With<StatRow>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }

    let Ok(list_entity) = list.single() else {
        return;
    };

    for stat in Stat::iter() {
        let Some(spans) = registry.get_snapshot_format(stat) else {
            continue;
        };
        let current = state.current.final_of(stat);
        let preview = state.preview.as_ref().map(|p| p.final_of(stat));

        let row = StatLineBuilder::spawn_line(
            &mut commands,
            spans,
            StatRenderMode::Snapshot { current, preview },
            ROW_FONT,
        );
        commands.entity(row).insert((ChildOf(list_entity), StatRow));
    }
}
