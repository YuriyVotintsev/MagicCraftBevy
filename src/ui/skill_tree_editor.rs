use bevy::camera::visibility::RenderLayers;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::skill_tree::graph::{GraphEdge, GraphNode, GridSettings, SkillGraph};
use crate::skill_tree::types::{Rarity, SkillTreeDefRaw, SkillTreeNodeRaw};
use crate::stats::StatRegistry;
use crate::stats::modifier_def::{ModifierDefRaw, StatRangeRaw};
use crate::wave::WavePhase;

use super::skill_tree_view::SkillTreeCamera;
use super::skill_tree_view::{SkillTreeNode, SkillTreeEdge, SkillTreeWorld};

const SKILL_TREE_LAYER: usize = 1;
const NODE_CLICK_RADIUS: f32 = 40.0;
const GRID_LINE_COLOR: Color = Color::srgba(0.2, 0.2, 0.3, 0.25);
const GRID_LINE_WIDTH: f32 = 1.0;
const SELECTED_OUTLINE_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const EDGE_PREVIEW_COLOR: Color = Color::srgba(0.8, 0.8, 0.2, 0.5);

#[derive(Resource, Default)]
pub struct EditorMode(pub bool);

#[derive(Resource, Default)]
struct EditorState {
    dragging: Option<usize>,
    drag_offset: Vec2,
    drag_moved: bool,
    selected: Option<usize>,
    edge_start: Option<usize>,
    last_node_count: usize,
    last_edge_count: usize,
}

#[derive(Clone, PartialEq)]
enum EditField {
    NodeName(usize),
    MaxLevel(usize),
    StatValue(usize, usize),
}

#[derive(Resource, Default)]
struct ActiveEdit {
    field: Option<EditField>,
    buffer: String,
}

#[derive(Component)]
struct EditorPanel;

#[derive(Component)]
struct EditorGridLines;

#[derive(Component)]
struct EditorSelectedOutline;

#[derive(Component)]
struct EditorEdgePreview;

#[derive(Component)]
struct SaveButton;

#[derive(Component)]
struct GridSizeInput;

#[derive(Component)]
struct NodePropertyPanel;

#[derive(Component)]
struct EditableField(EditField);

#[derive(Component)]
struct EditFieldText(EditField);

#[derive(Component)]
struct GridSizePlus;

#[derive(Component)]
struct GridSizeMinus;

#[derive(Component)]
struct AddStatBtn(usize);

#[derive(Component)]
struct RemoveStatBtn(usize, usize);

#[derive(Component)]
struct StatNameBtn(usize, usize);

#[derive(Component)]
struct CreateNodePopup;

#[derive(Component)]
struct CreateNodeBtn {
    position: Vec2,
    grid_cell: IVec2,
}

#[derive(Component)]
struct StatDropdown;

#[derive(Component)]
struct StatDropdownItem {
    node_idx: usize,
    stat_idx: usize,
    stat_id: crate::stats::StatId,
}

const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.15, 0.92);
const FIELD_BG: Color = Color::srgb(0.12, 0.12, 0.2);
const BUTTON_BG: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_HOVER: Color = Color::srgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);

pub struct SkillTreeEditorPlugin;

impl Plugin for SkillTreeEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorMode>()
            .init_resource::<EditorState>()
            .init_resource::<ActiveEdit>()
            .add_systems(
                Update,
                toggle_editor.run_if(in_state(WavePhase::Shop)),
            )
            .add_systems(
                Update,
                (
                    editor_node_drag,
                    editor_update_node_transforms,
                    editor_update_edge_transforms,
                    editor_edge_mode,
                    editor_delete,
                    editor_rebuild_world,
                    editor_save,
                    editor_text_input,
                )
                    .run_if(in_state(WavePhase::Shop))
                    .run_if(editor_active),
            )
            .add_systems(
                Update,
                (
                    editor_field_click,
                    editor_grid_size_buttons,
                    editor_stat_buttons,
                    editor_stat_name_click,
                    editor_stat_dropdown_select,
                    editor_create_node_confirm,
                    editor_dismiss_popups,
                    editor_update_grid_lines,
                    editor_update_selected_outline,
                    editor_update_edge_preview,
                    editor_panel_interactions,
                )
                    .run_if(in_state(WavePhase::Shop))
                    .run_if(editor_active),
            );
    }
}

fn editor_active(mode: Res<EditorMode>) -> bool {
    mode.0
}

fn toggle_editor(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor_mode: ResMut<EditorMode>,
    mut editor_state: ResMut<EditorState>,
    panel_query: Query<Entity, With<EditorPanel>>,
    grid_query: Query<Entity, With<EditorGridLines>>,
    outline_query: Query<Entity, With<EditorSelectedOutline>>,
    preview_query: Query<Entity, With<EditorEdgePreview>>,
    prop_query: Query<Entity, With<NodePropertyPanel>>,
    popup_query: Query<Entity, With<CreateNodePopup>>,
    dropdown_query: Query<Entity, With<StatDropdown>>,
    grid_settings: Option<Res<GridSettings>>,
) {
    if !keyboard.just_pressed(KeyCode::Backquote) {
        return;
    }

    editor_mode.0 = !editor_mode.0;
    *editor_state = EditorState::default();

    if editor_mode.0 {
        let grid_size = grid_settings.map(|g| g.size).unwrap_or(100.0);
        spawn_editor_panel(&mut commands, grid_size);
        info!("Skill tree editor enabled");
    } else {
        for entity in panel_query.iter()
            .chain(grid_query.iter())
            .chain(outline_query.iter())
            .chain(preview_query.iter())
            .chain(prop_query.iter())
            .chain(popup_query.iter())
            .chain(dropdown_query.iter())
        {
            commands.entity(entity).despawn();
        }
        info!("Skill tree editor disabled");
    }
}

fn spawn_editor_panel(commands: &mut Commands, grid_size: f32) {
    commands.spawn((
        EditorPanel,
        DespawnOnExit(WavePhase::Shop),
        GlobalZIndex(100),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(60.0),
            width: Val::Px(260.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        children![
            (
                Text::new("EDITOR"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(GOLD_COLOR),
            ),
            (
                Button,
                SaveButton,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
                children![(
                    Text::new("Save (Ctrl+S)"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(TEXT_COLOR),
                )]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                children![
                    (
                        Text::new("Grid:"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(TEXT_COLOR),
                    ),
                    (
                        Button,
                        GridSizeMinus,
                        Node {
                            width: Val::Px(24.0), height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center, ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                        children![(
                            Text::new("-"),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(TEXT_COLOR),
                        )]
                    ),
                    (
                        GridSizeInput,
                        Text(format!("{}", grid_size as i32)),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(GOLD_COLOR),
                        Node {
                            padding: UiRect::horizontal(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(FIELD_BG),
                    ),
                    (
                        Button,
                        GridSizePlus,
                        Node {
                            width: Val::Px(24.0), height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center, ..default()
                        },
                        BackgroundColor(BUTTON_BG),
                        children![(
                            Text::new("+"),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(TEXT_COLOR),
                        )]
                    ),
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                children![
                    (
                        Text::new("Shift+Click: edge"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                    (
                        Text::new("Del: remove node"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                    (
                        Text::new("Click empty: add node"),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ),
                ]
            ),
        ],
    ));
}

fn editor_node_drag(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SkillTreeCamera>>,
    mut graph: Option<ResMut<SkillGraph>>,
    grid_settings: Option<Res<GridSettings>>,
    mut editor_state: ResMut<EditorState>,
    mut node_query: Query<(&SkillTreeNode, &mut Transform)>,
    mut commands: Commands,
    prop_query: Query<Entity, With<NodePropertyPanel>>,
    stat_registry: Res<StatRegistry>,
) {
    let Some(ref mut graph) = graph else { return };
    let grid_size = grid_settings.as_ref().map(|g| g.size).unwrap_or(100.0);

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else { return };

    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
        return;
    }

    let find_node_at = |pos: Vec2, graph: &SkillGraph| -> Option<usize> {
        let mut best_idx = None;
        let mut best_dist = NODE_CLICK_RADIUS;
        for (idx, node) in graph.nodes.iter().enumerate() {
            let dist = pos.distance(node.position);
            if dist < best_dist {
                best_dist = dist;
                best_idx = Some(idx);
            }
        }
        best_idx
    };

    // LMB: drag nodes + select on click (no drag)
    if mouse.just_pressed(MouseButton::Left) && editor_state.dragging.is_none() {
        if let Some(idx) = find_node_at(world_pos, graph) {
            editor_state.dragging = Some(idx);
            editor_state.drag_offset = graph.nodes[idx].position - world_pos;
            editor_state.drag_moved = false;
        }
    }

    if let Some(idx) = editor_state.dragging {
        let offset = editor_state.drag_offset;
        if mouse.pressed(MouseButton::Left) {
            let new_pos = world_pos + offset;
            if new_pos.distance(graph.nodes[idx].position) > 2.0 {
                graph.nodes[idx].position = new_pos;
                for (stn, mut transform) in &mut node_query {
                    if stn.graph_index == idx {
                        transform.translation = new_pos.extend(1.0);
                    }
                }
                editor_state.drag_moved = true;
            }
        }

        if mouse.just_released(MouseButton::Left) {
            if editor_state.drag_moved {
                let snapped = snap_to_grid(world_pos + offset, grid_size);
                graph.nodes[idx].position = snapped;
                graph.nodes[idx].grid_cell = world_to_grid(snapped, grid_size);
                for (stn, mut transform) in &mut node_query {
                    if stn.graph_index == idx {
                        transform.translation = snapped.extend(1.0);
                    }
                }
            }

            if !editor_state.drag_moved {
                editor_state.selected = Some(idx);
                spawn_property_panel(&mut commands, graph, idx, &prop_query, &stat_registry);
            }

            editor_state.dragging = None;
            editor_state.drag_moved = false;
            graph.set_changed();
        }
    }

    // RMB on empty → show "Create Node" confirmation popup
    if mouse.just_pressed(MouseButton::Right) {
        if find_node_at(world_pos, graph).is_none() {
            let snapped = snap_to_grid(world_pos, grid_size);
            spawn_create_node_popup(&mut commands, cursor_pos, snapped, grid_size);
        }
    }
}

fn editor_update_node_transforms(
    graph: Option<Res<SkillGraph>>,
    mut node_query: Query<(&SkillTreeNode, &mut Transform)>,
) {
    let Some(graph) = graph else { return };
    if !graph.is_changed() {
        return;
    }
    for (stn, mut transform) in &mut node_query {
        if let Some(node) = graph.nodes.get(stn.graph_index) {
            transform.translation = node.position.extend(1.0);
        }
    }
}

fn editor_update_edge_transforms(
    graph: Option<Res<SkillGraph>>,
    mut edge_query: Query<(&SkillTreeEdge, &mut Transform)>,
) {
    let Some(graph) = graph else { return };
    if !graph.is_changed() {
        return;
    }

    for (edge_comp, mut transform) in &mut edge_query {
        if edge_comp.edge_index >= graph.edges.len() {
            continue;
        }
        let edge = &graph.edges[edge_comp.edge_index];
        if edge.a >= graph.nodes.len() || edge.b >= graph.nodes.len() {
            continue;
        }
        let pos_a = graph.nodes[edge.a].position;
        let pos_b = graph.nodes[edge.b].position;
        let center = (pos_a + pos_b) / 2.0;
        let diff = pos_b - pos_a;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x);
        *transform = Transform::from_translation(center.extend(0.0))
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(length, 2.5, 1.0));
    }
}

fn editor_edge_mode(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SkillTreeCamera>>,
    graph: Option<ResMut<SkillGraph>>,
    mut editor_state: ResMut<EditorState>,
) {
    if !(keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)) {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(mut graph) = graph else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else { return };

    let mut clicked_idx = None;
    let mut best_dist = NODE_CLICK_RADIUS;
    for (idx, node) in graph.nodes.iter().enumerate() {
        let dist = world_pos.distance(node.position);
        if dist < best_dist {
            best_dist = dist;
            clicked_idx = Some(idx);
        }
    }

    let Some(clicked) = clicked_idx else {
        editor_state.edge_start = None;
        return;
    };

    if let Some(start) = editor_state.edge_start {
        if start != clicked {
            let edge_exists = graph.edges.iter().any(|e|
                (e.a == start && e.b == clicked) || (e.a == clicked && e.b == start)
            );

            if edge_exists {
                graph.edges.retain(|e|
                    !((e.a == start && e.b == clicked) || (e.a == clicked && e.b == start))
                );
                graph.adjacency[start].retain(|&n| n != clicked);
                graph.adjacency[clicked].retain(|&n| n != start);
            } else {
                graph.edges.push(GraphEdge { a: start, b: clicked });
                graph.adjacency[start].push(clicked);
                graph.adjacency[clicked].push(start);
            }
            graph.set_changed();
        }
        editor_state.edge_start = None;
    } else {
        editor_state.edge_start = Some(clicked);
    }
}

fn editor_delete(
    keyboard: Res<ButtonInput<KeyCode>>,
    graph: Option<ResMut<SkillGraph>>,
    mut editor_state: ResMut<EditorState>,
    mut commands: Commands,
    panel_query: Query<Entity, With<NodePropertyPanel>>,
) {
    if !keyboard.just_pressed(KeyCode::Delete) {
        return;
    }

    let Some(mut graph) = graph else { return };
    let Some(idx) = editor_state.selected else { return };

    if idx == graph.start_node {
        return;
    }

    graph.nodes.remove(idx);
    graph.edges.retain(|e| e.a != idx && e.b != idx);

    for edge in &mut graph.edges {
        if edge.a > idx { edge.a -= 1; }
        if edge.b > idx { edge.b -= 1; }
    }

    let len = graph.nodes.len();
    let mut adjacency = vec![Vec::new(); len];
    for edge in &graph.edges {
        adjacency[edge.a].push(edge.b);
        adjacency[edge.b].push(edge.a);
    }
    graph.adjacency = adjacency;

    if graph.start_node > idx {
        graph.start_node -= 1;
    }

    editor_state.selected = None;
    for entity in &panel_query {
        commands.entity(entity).despawn();
    }
    graph.set_changed();
}

fn editor_rebuild_world(
    mut commands: Commands,
    graph: Option<Res<SkillGraph>>,
    mut editor_state: ResMut<EditorState>,
    world_query: Query<Entity, With<SkillTreeWorld>>,
    st_meshes: Option<Res<super::skill_tree_view::SkillTreeMeshes>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(graph) = graph else { return };
    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();

    if node_count == editor_state.last_node_count && edge_count == editor_state.last_edge_count {
        return;
    }
    editor_state.last_node_count = node_count;
    editor_state.last_edge_count = edge_count;

    for entity in &world_query {
        commands.entity(entity).despawn();
    }

    if let Some(st_meshes) = st_meshes {
        super::skill_tree_view::spawn_skill_tree_world(&mut commands, &graph, &st_meshes, &mut materials);
    }
}

fn editor_save(
    keyboard: Res<ButtonInput<KeyCode>>,
    save_btn: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    graph: Option<Res<SkillGraph>>,
    grid_settings: Option<Res<GridSettings>>,
    stat_registry: Res<StatRegistry>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let ctrl_s = ctrl && keyboard.just_pressed(KeyCode::KeyS);
    let btn_pressed = save_btn.iter().any(|i| *i == Interaction::Pressed);

    if !ctrl_s && !btn_pressed {
        return;
    }

    let Some(graph) = graph else { return };
    let grid_size = grid_settings.map(|g| g.size).unwrap_or(100.0);

    let raw = graph_to_raw(&graph, &stat_registry, grid_size);
    let config = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .new_line("\n".to_string());

    match ron::ser::to_string_pretty(&raw, config) {
        Ok(content) => {
            let path = "assets/skill_tree/tree.ron";
            match std::fs::write(path, &content) {
                Ok(_) => info!("Saved skill tree to {}", path),
                Err(e) => error!("Failed to save skill tree: {}", e),
            }
        }
        Err(e) => error!("Failed to serialize skill tree: {}", e),
    }
}

fn graph_to_raw(graph: &SkillGraph, stat_registry: &StatRegistry, grid_size: f32) -> SkillTreeDefRaw {
    let nodes = graph.nodes.iter().map(|node| {
        let modifiers = if node.rolled_values.is_empty() {
            vec![]
        } else {
            vec![ModifierDefRaw {
                stats: node.rolled_values.iter().map(|(stat_id, value)| {
                    StatRangeRaw::Fixed {
                        stat: stat_registry.name(*stat_id).unwrap_or("unknown").to_string(),
                        value: *value,
                    }
                }).collect()
            }]
        };
        SkillTreeNodeRaw {
            name: node.name.clone(),
            position: (node.grid_cell.x, node.grid_cell.y),
            max_level: node.max_level,
            modifiers,
        }
    }).collect();

    let edges = graph.edges.iter().map(|e| (e.a, e.b)).collect();
    SkillTreeDefRaw { grid_size, nodes, edges }
}

fn editor_update_grid_lines(
    mut commands: Commands,
    camera_query: Query<(&Transform, &Projection), With<SkillTreeCamera>>,
    grid_settings: Option<Res<GridSettings>>,
    editor_mode: Res<EditorMode>,
    old_lines: Query<Entity, With<EditorGridLines>>,
    st_meshes: Option<Res<super::skill_tree_view::SkillTreeMeshes>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in &old_lines {
        commands.entity(entity).despawn();
    }

    if !editor_mode.0 {
        return;
    }

    let grid_size = grid_settings.map(|g| g.size).unwrap_or(100.0);
    let Some(st_meshes) = st_meshes else { return };

    let Ok((cam_transform, projection)) = camera_query.single() else { return };
    let cam_pos = cam_transform.translation.truncate();
    let scale = cam_transform.scale.x;

    let half_view = match projection {
        Projection::Orthographic(ortho) => match ortho.scaling_mode {
            bevy::camera::ScalingMode::FixedVertical { viewport_height } => {
                Vec2::new(viewport_height * 1.78 * scale, viewport_height * scale)
            }
            _ => Vec2::new(1920.0 * scale, 1080.0 * scale),
        },
        _ => Vec2::new(1920.0 * scale, 1080.0 * scale),
    };

    let margin = grid_size * 2.0;
    let min_x = ((cam_pos.x - half_view.x - margin) / grid_size).floor() as i32;
    let max_x = ((cam_pos.x + half_view.x + margin) / grid_size).ceil() as i32;
    let min_y = ((cam_pos.y - half_view.y - margin) / grid_size).floor() as i32;
    let max_y = ((cam_pos.y + half_view.y + margin) / grid_size).ceil() as i32;

    let render_layer = RenderLayers::layer(SKILL_TREE_LAYER);
    let line_mat = materials.add(ColorMaterial::from_color(GRID_LINE_COLOR));
    let total_h = (max_y - min_y) as f32 * grid_size;
    let total_w = (max_x - min_x) as f32 * grid_size;
    let center_y = (min_y as f32 + max_y as f32) / 2.0 * grid_size;
    let center_x = (min_x as f32 + max_x as f32) / 2.0 * grid_size;

    let parent = commands.spawn((
        EditorGridLines,
        Transform::default(),
        Visibility::Inherited,
    )).id();

    for x in min_x..=max_x {
        let wx = x as f32 * grid_size;
        let child = commands.spawn((
            Mesh2d(st_meshes.rect.clone()),
            MeshMaterial2d(line_mat.clone()),
            Transform::from_translation(Vec3::new(wx, center_y, -5.0))
                .with_scale(Vec3::new(GRID_LINE_WIDTH, total_h, 1.0)),
            render_layer.clone(),
        )).id();
        commands.entity(parent).add_child(child);
    }
    for y in min_y..=max_y {
        let wy = y as f32 * grid_size;
        let child = commands.spawn((
            Mesh2d(st_meshes.rect.clone()),
            MeshMaterial2d(line_mat.clone()),
            Transform::from_translation(Vec3::new(center_x, wy, -5.0))
                .with_scale(Vec3::new(total_w, GRID_LINE_WIDTH, 1.0)),
            render_layer.clone(),
        )).id();
        commands.entity(parent).add_child(child);
    }
}

fn editor_update_selected_outline(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    graph: Option<Res<SkillGraph>>,
    old_outline: Query<Entity, With<EditorSelectedOutline>>,
    st_meshes: Option<Res<super::skill_tree_view::SkillTreeMeshes>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in &old_outline {
        commands.entity(entity).despawn();
    }

    let Some(graph) = graph else { return };
    let Some(st_meshes) = st_meshes else { return };
    let Some(idx) = editor_state.selected else { return };
    let Some(node) = graph.nodes.get(idx) else { return };

    let render_layer = RenderLayers::layer(SKILL_TREE_LAYER);
    let mat = materials.add(ColorMaterial::from_color(SELECTED_OUTLINE_COLOR));

    commands.spawn((
        EditorSelectedOutline,
        Mesh2d(st_meshes.circle.clone()),
        MeshMaterial2d(mat),
        Transform::from_translation(node.position.extend(0.5))
            .with_scale(Vec3::splat(30.0)),
        render_layer,
    ));
}

fn editor_update_edge_preview(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    graph: Option<Res<SkillGraph>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SkillTreeCamera>>,
    old_preview: Query<Entity, With<EditorEdgePreview>>,
    st_meshes: Option<Res<super::skill_tree_view::SkillTreeMeshes>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in &old_preview {
        commands.entity(entity).despawn();
    }

    let Some(start_idx) = editor_state.edge_start else { return };
    let Some(graph) = graph else { return };
    let Some(st_meshes) = st_meshes else { return };
    let Some(start_node) = graph.nodes.get(start_idx) else { return };

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else { return };

    let pos_a = start_node.position;
    let center = (pos_a + world_pos) / 2.0;
    let diff = world_pos - pos_a;
    let length = diff.length();
    let angle = diff.y.atan2(diff.x);

    let render_layer = RenderLayers::layer(SKILL_TREE_LAYER);
    let mat = materials.add(ColorMaterial::from_color(EDGE_PREVIEW_COLOR));

    commands.spawn((
        EditorEdgePreview,
        Mesh2d(st_meshes.rect.clone()),
        MeshMaterial2d(mat),
        Transform::from_translation(center.extend(0.5))
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(length, 2.5, 1.0)),
        render_layer,
    ));
}

fn editor_panel_interactions(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, Or<(With<SaveButton>, With<GridSizePlus>, With<GridSizeMinus>, With<AddStatBtn>, With<RemoveStatBtn>)>)>,
) {
    for (interaction, mut color) in &mut buttons {
        match interaction {
            Interaction::Hovered => *color = BUTTON_HOVER.into(),
            Interaction::None => *color = BUTTON_BG.into(),
            _ => {}
        }
    }
}

fn editor_field_click(
    interaction_query: Query<(&Interaction, &EditableField), Changed<Interaction>>,
    mut active_edit: ResMut<ActiveEdit>,
    graph: Option<Res<SkillGraph>>,
    mut text_query: Query<(&EditFieldText, &mut Text, &mut TextColor)>,
) {
    for (interaction, editable) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let field = editable.0.clone();
        let current_value = match &field {
            EditField::NodeName(idx) => {
                graph.as_ref().and_then(|g| g.nodes.get(*idx)).map(|n| n.name.clone()).unwrap_or_default()
            }
            EditField::MaxLevel(idx) => {
                graph.as_ref().and_then(|g| g.nodes.get(*idx)).map(|n| format!("{}", n.max_level)).unwrap_or_default()
            }
            EditField::StatValue(node_idx, stat_idx) => {
                graph.as_ref()
                    .and_then(|g| g.nodes.get(*node_idx))
                    .and_then(|n| n.rolled_values.get(*stat_idx))
                    .map(|(_, v)| format!("{}", v))
                    .unwrap_or_default()
            }
        };
        active_edit.field = Some(field);
        active_edit.buffer = current_value;
    }

    if let Some(ref field) = active_edit.field {
        for (ft, mut text, mut color) in &mut text_query {
            if ft.0 == *field {
                text.0 = format!(">{}<", active_edit.buffer);
                *color = TextColor(GOLD_COLOR);
            }
        }
    }
}

fn editor_text_input(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut active_edit: ResMut<ActiveEdit>,
    mut graph: Option<ResMut<SkillGraph>>,
    mut commands: Commands,
    editor_state: Res<EditorState>,
    prop_query: Query<Entity, With<NodePropertyPanel>>,
    stat_registry: Res<StatRegistry>,
) {
    if active_edit.field.is_none() {
        return;
    }

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        match &event.logical_key {
            Key::Character(c) => {
                active_edit.buffer.push_str(c.as_str());
            }
            Key::Backspace => {
                active_edit.buffer.pop();
            }
            Key::Enter => {
                let field = active_edit.field.take().unwrap();
                let buf = std::mem::take(&mut active_edit.buffer);
                apply_edit(&field, &buf, &mut graph);
                if let (Some(idx), Some(ref graph)) = (editor_state.selected, &graph) {
                    spawn_property_panel(&mut commands, graph, idx, &prop_query, &stat_registry);
                }
            }
            Key::Escape => {
                active_edit.field = None;
                active_edit.buffer.clear();
                if let (Some(idx), Some(ref graph)) = (editor_state.selected, &graph) {
                    spawn_property_panel(&mut commands, graph, idx, &prop_query, &stat_registry);
                }
            }
            Key::Space => {
                active_edit.buffer.push(' ');
            }
            _ => {}
        }
    }
}

fn apply_edit(
    field: &EditField,
    value: &str,
    graph: &mut Option<ResMut<SkillGraph>>,
) {
    match field {
        EditField::NodeName(idx) => {
            if let Some(ref mut graph) = graph {
                if let Some(node) = graph.nodes.get_mut(*idx) {
                    node.name = value.to_string();
                }
            }
        }
        EditField::MaxLevel(idx) => {
            if let Ok(v) = value.parse::<u32>() {
                let v = v.max(1);
                if let Some(ref mut graph) = graph {
                    if let Some(node) = graph.nodes.get_mut(*idx) {
                        node.max_level = v;
                    }
                }
            }
        }
        EditField::StatValue(node_idx, stat_idx) => {
            if let Ok(v) = value.parse::<f32>() {
                if let Some(ref mut graph) = graph {
                    if let Some(node) = graph.nodes.get_mut(*node_idx) {
                        if let Some(entry) = node.rolled_values.get_mut(*stat_idx) {
                            entry.1 = v;
                        }
                    }
                }
            }
        }
    }
}

fn editor_grid_size_buttons(
    plus_query: Query<&Interaction, (Changed<Interaction>, With<GridSizePlus>)>,
    minus_query: Query<&Interaction, (Changed<Interaction>, With<GridSizeMinus>)>,
    mut grid_settings: Option<ResMut<GridSettings>>,
    mut graph: Option<ResMut<SkillGraph>>,
    mut grid_text: Query<&mut Text, With<GridSizeInput>>,
    mut node_query: Query<(&SkillTreeNode, &mut Transform)>,
) {
    let mut delta = 0.0_f32;
    for interaction in &plus_query {
        if *interaction == Interaction::Pressed { delta += 25.0; }
    }
    for interaction in &minus_query {
        if *interaction == Interaction::Pressed { delta -= 25.0; }
    }
    if delta == 0.0 { return; }

    let Some(ref mut gs) = grid_settings else { return };
    gs.size = (gs.size + delta).clamp(25.0, 500.0);
    let new_size = gs.size;

    if let Some(ref mut graph) = graph {
        for node in &mut graph.nodes {
            node.position = Vec2::new(
                node.grid_cell.x as f32 * new_size,
                node.grid_cell.y as f32 * new_size,
            );
        }
        for (stn, mut transform) in &mut node_query {
            if let Some(node) = graph.nodes.get(stn.graph_index) {
                transform.translation = node.position.extend(1.0);
            }
        }
        graph.set_changed();
    }
    for mut text in &mut grid_text {
        text.0 = format!("{}", new_size as i32);
    }
}

fn editor_stat_buttons(
    add_query: Query<(&Interaction, &AddStatBtn), Changed<Interaction>>,
    remove_query: Query<(&Interaction, &RemoveStatBtn), Changed<Interaction>>,
    mut graph: Option<ResMut<SkillGraph>>,
    stat_registry: Res<StatRegistry>,
    mut commands: Commands,
    editor_state: Res<EditorState>,
    prop_query: Query<Entity, With<NodePropertyPanel>>,
) {
    let Some(ref mut graph) = graph else { return };
    let mut rebuild = false;

    for (interaction, add) in &add_query {
        if *interaction != Interaction::Pressed { continue; }
        if let Some(node) = graph.nodes.get_mut(add.0) {
            if let Some(default_stat) = stat_registry.get("max_stamina_flat") {
                node.rolled_values.push((default_stat, 1.0));
                rebuild = true;
            }
        }
    }

    for (interaction, remove) in &remove_query {
        if *interaction != Interaction::Pressed { continue; }
        if let Some(node) = graph.nodes.get_mut(remove.0) {
            if remove.1 < node.rolled_values.len() {
                node.rolled_values.remove(remove.1);
                rebuild = true;
            }
        }
    }

    if rebuild {
        if let Some(idx) = editor_state.selected {
            spawn_property_panel(&mut commands, graph, idx, &prop_query, &stat_registry);
        }
    }
}

const EDIT_FIELD_BG: Color = Color::srgba(0.1, 0.1, 0.18, 1.0);
const DROPDOWN_BG: Color = Color::srgb(0.08, 0.08, 0.14);
const DROPDOWN_ITEM_BG: Color = Color::srgb(0.1, 0.1, 0.18);
const DROPDOWN_ITEM_HOVER: Color = Color::srgb(0.18, 0.18, 0.3);

fn spawn_property_panel(
    commands: &mut Commands,
    graph: &SkillGraph,
    idx: usize,
    panel_query: &Query<Entity, With<NodePropertyPanel>>,
    stat_registry: &StatRegistry,
) {
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }

    let Some(node) = graph.nodes.get(idx) else { return };

    let mut children_list: Vec<Entity> = Vec::new();

    children_list.push(commands.spawn((
        Text(format!("Node #{} ({}, {})", idx, node.grid_cell.x, node.grid_cell.y)),
        TextFont { font_size: 15.0, ..default() },
        TextColor(GOLD_COLOR),
        Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
    )).id());

    // Name field (click to edit)
    children_list.push(spawn_editable_row(
        commands, "Name", &node.name, EditField::NodeName(idx),
    ));

    children_list.push(spawn_editable_row(
        commands, "Max Level", &format!("{}", node.max_level), EditField::MaxLevel(idx),
    ));

    // Stats
    children_list.push(commands.spawn((
        Text::new("Stats:"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(TEXT_COLOR),
        Node { margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(2.0)), ..default() },
    )).id());

    for (si, (stat_id, value)) in node.rolled_values.iter().enumerate() {
        let stat_name = stat_registry.name(*stat_id).unwrap_or("?");
        let row = commands.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        }).id();

        let label = commands.spawn((
            Button,
            Interaction::default(),
            StatNameBtn(idx, si),
            Text(format!("{}:", stat_name)),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.6, 0.7, 0.6)),
            Node {
                padding: UiRect::horizontal(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(EDIT_FIELD_BG),
        )).id();

        let val_field = commands.spawn((
            Button,
            Interaction::default(),
            EditableField(EditField::StatValue(idx, si)),
            EditFieldText(EditField::StatValue(idx, si)),
            Text(format!("{}", value)),
            TextFont { font_size: 13.0, ..default() },
            TextColor(TEXT_COLOR),
            Node {
                padding: UiRect::horizontal(Val::Px(4.0)),
                min_width: Val::Px(50.0),
                ..default()
            },
            BackgroundColor(EDIT_FIELD_BG),
        )).id();

        let del_btn = commands.spawn((
            Button,
            RemoveStatBtn(idx, si),
            Node {
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.4, 0.15, 0.15)),
            children![(
                Text::new("x"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(TEXT_COLOR),
            )]
        )).id();

        commands.entity(row).add_children(&[label, val_field, del_btn]);
        children_list.push(row);
    }

    {
        children_list.push(commands.spawn((
            Button,
            AddStatBtn(idx),
            Node {
                height: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(4.0)),
                padding: UiRect::horizontal(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            children![(
                Text::new("+ Add Stat"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(TEXT_COLOR),
            )]
        )).id());
    }

    let panel = commands.spawn((
        NodePropertyPanel,
        DespawnOnExit(WavePhase::Shop),
        GlobalZIndex(100),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(340.0),
            width: Val::Px(260.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(2.0),
            ..default()
        },
        BackgroundColor(PANEL_BG),
    )).id();

    for child in children_list {
        commands.entity(panel).add_child(child);
    }
}

fn spawn_editable_row(commands: &mut Commands, label: &str, value: &str, field: EditField) -> Entity {
    let row = commands.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(4.0),
        margin: UiRect::bottom(Val::Px(3.0)),
        ..default()
    }).id();

    let lbl = commands.spawn((
        Text(format!("{}:", label)),
        TextFont { font_size: 14.0, ..default() },
        TextColor(TEXT_COLOR),
    )).id();

    let val = commands.spawn((
        Button,
        Interaction::default(),
        EditableField(field.clone()),
        EditFieldText(field),
        Text(value.to_string()),
        TextFont { font_size: 14.0, ..default() },
        TextColor(TEXT_COLOR),
        Node {
            padding: UiRect::horizontal(Val::Px(6.0)),
            min_width: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(EDIT_FIELD_BG),
    )).id();

    commands.entity(row).add_children(&[lbl, val]);
    row
}

fn editor_stat_name_click(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &StatNameBtn), Changed<Interaction>>,
    stat_registry: Res<StatRegistry>,
    old_dropdown: Query<Entity, With<StatDropdown>>,
) {
    for (interaction, stat_btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        for entity in &old_dropdown {
            commands.entity(entity).despawn();
        }

        let node_idx = stat_btn.0;
        let stat_idx = stat_btn.1;

        let mut items: Vec<Entity> = Vec::new();
        for (stat_id, def) in stat_registry.iter() {
            let item = commands.spawn((
                Button,
                Interaction::default(),
                StatDropdownItem { node_idx, stat_idx, stat_id },
                Text(def.name.clone()),
                TextFont { font_size: 12.0, ..default() },
                TextColor(TEXT_COLOR),
                Node {
                    padding: UiRect::all(Val::Px(4.0)),
                    width: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(DROPDOWN_ITEM_BG),
            )).id();
            items.push(item);
        }

        let dropdown = commands.spawn((
            StatDropdown,
            DespawnOnExit(WavePhase::Shop),
            GlobalZIndex(200),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(280.0),
                top: Val::Px(340.0),
                width: Val::Px(220.0),
                max_height: Val::Px(500.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(DROPDOWN_BG),
        )).id();

        for item in items {
            commands.entity(dropdown).add_child(item);
        }
    }
}

fn editor_stat_dropdown_select(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &StatDropdownItem), Changed<Interaction>>,
    mut graph: Option<ResMut<SkillGraph>>,
    stat_registry: Res<StatRegistry>,
    dropdown_query: Query<Entity, With<StatDropdown>>,
    editor_state: Res<EditorState>,
    prop_query: Query<Entity, With<NodePropertyPanel>>,
    mut hover_query: Query<(&Interaction, &mut BackgroundColor), (With<StatDropdownItem>, Without<StatNameBtn>)>,
) {
    for (interaction, bg) in &mut hover_query {
        match interaction {
            Interaction::Hovered => { *bg.into_inner() = DROPDOWN_ITEM_HOVER.into(); }
            Interaction::None => { *bg.into_inner() = DROPDOWN_ITEM_BG.into(); }
            _ => {}
        }
    }

    for (interaction, item) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Some(ref mut graph) = graph {
            if let Some(node) = graph.nodes.get_mut(item.node_idx) {
                if let Some(entry) = node.rolled_values.get_mut(item.stat_idx) {
                    entry.0 = item.stat_id;
                }
            }
        }

        for entity in &dropdown_query {
            commands.entity(entity).despawn();
        }

        if let (Some(idx), Some(ref graph)) = (editor_state.selected, &graph) {
            spawn_property_panel(&mut commands, graph, idx, &prop_query, &stat_registry);
        }
    }
}

fn spawn_create_node_popup(commands: &mut Commands, cursor_pos: Vec2, snapped: Vec2, grid_size: f32) {
    let grid_cell = IVec2::new(
        (snapped.x / grid_size).round() as i32,
        (snapped.y / grid_size).round() as i32,
    );

    commands.spawn((
        CreateNodePopup,
        DespawnOnExit(WavePhase::Shop),
        GlobalZIndex(150),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(cursor_pos.x),
            top: Val::Px(cursor_pos.y),
            ..default()
        },
        children![(
            Button,
            CreateNodeBtn { position: snapped, grid_cell },
            Node {
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(BUTTON_BG),
            children![(
                Text(format!("Create Node ({}, {})", grid_cell.x, grid_cell.y)),
                TextFont { font_size: 14.0, ..default() },
                TextColor(TEXT_COLOR),
            )]
        )],
    ));
}

fn editor_create_node_confirm(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &CreateNodeBtn), Changed<Interaction>>,
    mut graph: Option<ResMut<SkillGraph>>,
    mut editor_state: ResMut<EditorState>,
    popup_query: Query<Entity, With<CreateNodePopup>>,
) {
    for (interaction, btn) in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(ref mut graph) = graph else { continue };
        let new_idx = graph.nodes.len();

        graph.nodes.push(GraphNode {
            position: btn.position,
            grid_cell: btn.grid_cell,
            rarity: Rarity(0),
            rolled_values: Vec::new(),
            level: 0,
            max_level: 1,
            name: format!("Node {}", new_idx),
        });
        graph.adjacency.push(Vec::new());
        editor_state.selected = Some(new_idx);
        graph.set_changed();

        for entity in &popup_query {
            commands.entity(entity).despawn();
        }
    }
}

fn editor_dismiss_popups(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    popup_query: Query<Entity, With<CreateNodePopup>>,
    dropdown_query: Query<Entity, With<StatDropdown>>,
    popup_btn: Query<&Interaction, With<CreateNodeBtn>>,
    dropdown_btn: Query<&Interaction, With<StatDropdownItem>>,
) {
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if popup_query.is_empty() && dropdown_query.is_empty() {
        return;
    }

    let clicking_popup = popup_btn.iter().any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);
    let clicking_dropdown = dropdown_btn.iter().any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);

    if !clicking_popup {
        for entity in &popup_query {
            commands.entity(entity).despawn();
        }
    }
    if !clicking_dropdown {
        for entity in &dropdown_query {
            commands.entity(entity).despawn();
        }
    }
}

fn snap_to_grid(pos: Vec2, grid_size: f32) -> Vec2 {
    Vec2::new(
        (pos.x / grid_size).round() * grid_size,
        (pos.y / grid_size).round() * grid_size,
    )
}

fn world_to_grid(pos: Vec2, grid_size: f32) -> IVec2 {
    IVec2::new(
        (pos.x / grid_size).round() as i32,
        (pos.y / grid_size).round() as i32,
    )
}
