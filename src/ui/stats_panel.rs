use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::actors::compute_player_stats;
use crate::coord;
use crate::palette;
use crate::rune::{
    find_drop_target_world, Dragging, RuneGrid, RuneSource,
};
use crate::stats::{ComputedStats, Stat, StatCalculators, StatDisplayRegistry};

use super::stat_line_builder::{StatLineBuilder, StatRenderMode};
use super::widgets::panel_node;

const PANEL_LEFT: f32 = 40.0;
const PANEL_TOP: f32 = 100.0;
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
pub struct StatsPanelRoot;

#[derive(Component)]
pub struct StatsPanelList;

#[derive(Component)]
pub struct StatRow;

pub fn spawn_stats_panel(mut commands: Commands) {
    let panel = commands
        .spawn((
            StatsPanelRoot,
            DespawnOnExit(crate::wave::WavePhase::Shop),
            GlobalZIndex(50),
            panel_node(
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(PANEL_LEFT),
                    top: Val::Px(PANEL_TOP),
                    width: Val::Px(PANEL_WIDTH),
                    padding: UiRect::all(Val::Px(PANEL_PADDING)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                None,
            ),
        ))
        .id();

    commands.spawn((
        ChildOf(panel),
        Text::new("Stats"),
        TextFont { font_size: HEADER_FONT, ..default() },
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
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    dragging: Query<&Dragging>,
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
    let Ok((camera, transform)) = camera_q.single() else { return };
    let Some(cursor) = coord::cursor_ground_pos(window, camera, transform) else { return };
    let center = Vec3::new(cursor.x + d.grab_offset.x, 0.0, cursor.z + d.grab_offset.z);
    let Some(target) = find_drop_target_world(center, d.rune.is_joker(), &grid) else {
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
