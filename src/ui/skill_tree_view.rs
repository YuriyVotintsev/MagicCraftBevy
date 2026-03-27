use bevy::camera::ScalingMode;
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::balance::GameBalance;
use crate::money::PlayerMoney;
use crate::run::RunState;
use crate::skill_tree::graph::SkillGraph;
use crate::skill_tree::systems::AllocateNodeRequest;
use crate::skill_tree::types::{PassiveNodePool, Rarity};
use crate::stats::StatDisplayRegistry;
use crate::ui::stat_line_builder::{StatLineBuilder, StatRenderMode, GOLD_COLOR};
use crate::wave::WavePhase;

const SKILL_TREE_LAYER: usize = 1;

const COMMON_COLOR: Color = Color::srgb(0.63, 0.63, 0.63);
const UNCOMMON_COLOR: Color = Color::srgb(0.25, 0.75, 0.25);
const RARE_COLOR: Color = Color::srgb(0.25, 0.5, 1.0);
const EPIC_COLOR: Color = Color::srgb(0.63, 0.25, 0.88);
const START_NODE_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);

const RARITY_RADII: [f32; 4] = [16.0, 20.0, 26.0, 32.0];
const START_NODE_RADIUS: f32 = 24.0;

const LOCKED_BRIGHTNESS: f32 = 0.3;
const AVAILABLE_BRIGHTNESS: f32 = 0.8;
const ALLOCATED_BRIGHTNESS: f32 = 1.4;

const EDGE_WIDTH: f32 = 2.5;
const EDGE_DIM_ALPHA: f32 = 0.15;
const EDGE_MEDIUM_ALPHA: f32 = 0.5;
const EDGE_BRIGHT_ALPHA: f32 = 0.9;

const BG_COLOR: Color = Color::srgb(0.03, 0.03, 0.08);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const TOOLTIP_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.95);

const PAN_SPEED: f32 = 1.0;
const ZOOM_SPEED: f32 = 0.15;
const ZOOM_MIN: f32 = 0.3;
const ZOOM_MAX: f32 = 3.0;
const NODE_CLICK_RADIUS: f32 = 40.0;

#[derive(Component)]
pub struct SkillTreeCamera;

#[derive(Component)]
pub struct SkillTreeNode {
    pub graph_index: usize,
}

#[derive(Component)]
pub struct SkillTreeEdge {
    pub edge_index: usize,
}

#[derive(Component)]
pub struct SkillTreeWorld;

#[derive(Component)]
pub struct SkillTreeOverlay;

#[derive(Component)]
pub struct ShopCoinsText;

#[derive(Component)]
pub struct SkillTreeTooltip;

#[derive(Component)]
pub struct StartRunButton;

#[derive(Resource, Default)]
pub(crate) struct PanState {
    dragging: bool,
    last_pos: Vec2,
}

#[derive(Resource)]
pub(crate) struct ZoomLevel(f32);

impl Default for ZoomLevel {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Resource)]
pub(crate) struct SkillTreeMeshes {
    circle: Handle<Mesh>,
    rect: Handle<Mesh>,
}

fn rarity_color(rarity: Rarity) -> Color {
    match rarity.0 {
        0 => COMMON_COLOR,
        1 => UNCOMMON_COLOR,
        2 => RARE_COLOR,
        3 => EPIC_COLOR,
        _ => COMMON_COLOR,
    }
}

fn rarity_radius(rarity: Rarity) -> f32 {
    RARITY_RADII.get(rarity.0 as usize).copied().unwrap_or(16.0)
}

fn node_color(graph: &SkillGraph, idx: usize) -> Color {
    let node = &graph.nodes[idx];
    if node.is_start() {
        return START_NODE_COLOR;
    }

    let base_color = rarity_color(node.rarity);
    let brightness = if node.allocated {
        ALLOCATED_BRIGHTNESS
    } else if graph.is_allocatable(idx) {
        AVAILABLE_BRIGHTNESS
    } else {
        LOCKED_BRIGHTNESS
    };

    let [r, g, b, a] = base_color.to_srgba().to_f32_array();
    Color::srgba(r * brightness, g * brightness, b * brightness, a)
}

fn edge_alpha(graph: &SkillGraph, edge_idx: usize) -> f32 {
    let edge = &graph.edges[edge_idx];
    let a_allocated = graph.nodes[edge.a].allocated;
    let b_allocated = graph.nodes[edge.b].allocated;
    let a_available = graph.is_allocatable(edge.a);
    let b_available = graph.is_allocatable(edge.b);

    if a_allocated && b_allocated {
        EDGE_BRIGHT_ALPHA
    } else if (a_allocated && b_available) || (b_allocated && a_available) {
        EDGE_MEDIUM_ALPHA
    } else {
        EDGE_DIM_ALPHA
    }
}

pub fn setup_skill_tree_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let circle = meshes.add(Circle::new(1.0));
    let rect = meshes.add(Rectangle::new(1.0, 1.0));
    commands.insert_resource(SkillTreeMeshes { circle, rect });
}

pub fn spawn_shop_screen(
    mut commands: Commands,
    graph: Option<Res<SkillGraph>>,
    pool: Option<Res<PassiveNodePool>>,
    st_meshes: Option<Res<SkillTreeMeshes>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut pan_state: ResMut<PanState>,
    mut zoom_level: ResMut<ZoomLevel>,
    run_state: Res<RunState>,
    money: Res<PlayerMoney>,
    balance: Res<GameBalance>,
) {
    *pan_state = PanState::default();
    *zoom_level = ZoomLevel::default();

    commands.spawn((
        SkillTreeCamera,
        DespawnOnExit(WavePhase::Shop),
        Camera2d,
        Camera {
            order: 10,
            clear_color: ClearColorConfig::Custom(BG_COLOR),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(SKILL_TREE_LAYER),
    ));

    if let (Some(graph), Some(_pool), Some(st_meshes)) = (graph, pool, st_meshes) {
        spawn_skill_tree_world(&mut commands, &graph, &st_meshes, &mut materials);
    }

    commands.spawn((
        SkillTreeOverlay,
        DespawnOnExit(WavePhase::Shop),
        GlobalZIndex(50),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(24.0),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text(format!("Run {}", run_state.attempt)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ),
            (
                ShopCoinsText,
                Text(format!("Coins: {} (node: {}g)", money.get(), balance.run.node_cost)),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(GOLD_COLOR),
            ),
        ],
    ));

    commands.spawn((
        DespawnOnExit(WavePhase::Shop),
        GlobalZIndex(50),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(30.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            StartRunButton,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new("Start Run"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            )]
        )],
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

fn spawn_skill_tree_world(
    commands: &mut Commands,
    graph: &SkillGraph,
    st_meshes: &SkillTreeMeshes,
    materials: &mut Assets<ColorMaterial>,
) {
    let render_layer = RenderLayers::layer(SKILL_TREE_LAYER);

    let world = commands
        .spawn((
            SkillTreeWorld,
            Transform::default(),
            Visibility::Inherited,
        ))
        .id();

    let bg_mat = materials.add(ColorMaterial::from_color(BG_COLOR));
    let bg_entity = commands
        .spawn((
            Mesh2d(st_meshes.rect.clone()),
            MeshMaterial2d(bg_mat),
            Transform::from_translation(Vec3::new(0.0, 0.0, -10.0))
                .with_scale(Vec3::new(10000.0, 10000.0, 1.0)),
            render_layer.clone(),
        ))
        .id();
    commands.entity(world).add_child(bg_entity);

    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        let pos_a = graph.nodes[edge.a].position;
        let pos_b = graph.nodes[edge.b].position;
        let center = (pos_a + pos_b) / 2.0;
        let diff = pos_b - pos_a;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x);
        let alpha = edge_alpha(graph, edge_idx);

        let mat = materials.add(ColorMaterial::from_color(Color::srgba(
            0.5, 0.5, 0.6, alpha,
        )));

        let edge_entity = commands
            .spawn((
                SkillTreeEdge { edge_index: edge_idx },
                Mesh2d(st_meshes.rect.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(center.extend(0.0))
                    .with_rotation(Quat::from_rotation_z(angle))
                    .with_scale(Vec3::new(length, EDGE_WIDTH, 1.0)),
                render_layer.clone(),
            ))
            .id();

        commands.entity(world).add_child(edge_entity);
    }

    for (idx, node) in graph.nodes.iter().enumerate() {
        let color = node_color(graph, idx);
        let radius = if node.is_start() {
            START_NODE_RADIUS
        } else {
            rarity_radius(node.rarity)
        };

        let mat = materials.add(ColorMaterial::from_color(color));

        let node_entity = commands
            .spawn((
                SkillTreeNode { graph_index: idx },
                Mesh2d(st_meshes.circle.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(node.position.extend(1.0))
                    .with_scale(Vec3::splat(radius)),
                render_layer.clone(),
            ))
            .id();

        commands.entity(world).add_child(node_entity);
    }
}

pub fn skill_tree_pan_zoom(
    mouse: Res<ButtonInput<MouseButton>>,
    mut scroll_events: MessageReader<MouseWheel>,
    windows: Query<&Window>,
    mut camera_query: Query<&mut Transform, With<SkillTreeCamera>>,
    mut pan_state: ResMut<PanState>,
    mut zoom_level: ResMut<ZoomLevel>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        pan_state.dragging = false;
        return;
    };

    let Ok(mut cam_transform) = camera_query.single_mut() else {
        return;
    };

    if mouse.just_pressed(MouseButton::Right) || mouse.just_pressed(MouseButton::Middle) {
        pan_state.dragging = true;
        pan_state.last_pos = cursor_pos;
    }
    if mouse.just_released(MouseButton::Right) || mouse.just_released(MouseButton::Middle) {
        pan_state.dragging = false;
    }

    if pan_state.dragging {
        let delta = cursor_pos - pan_state.last_pos;
        cam_transform.translation.x -= delta.x * PAN_SPEED * zoom_level.0;
        cam_transform.translation.y += delta.y * PAN_SPEED * zoom_level.0;
        pan_state.last_pos = cursor_pos;
    }

    for event in scroll_events.read() {
        let zoom_delta = -event.y * ZOOM_SPEED;
        zoom_level.0 = (zoom_level.0 + zoom_delta).clamp(ZOOM_MIN, ZOOM_MAX);
        cam_transform.scale = Vec3::splat(zoom_level.0);
    }
}

pub fn skill_tree_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SkillTreeCamera>>,
    graph: Option<Res<SkillGraph>>,
    money: Res<PlayerMoney>,
    balance: Res<GameBalance>,
    mut allocate_events: MessageWriter<AllocateNodeRequest>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(graph) = graph else { return };
    if !money.can_afford(balance.run.node_cost) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_gt)) = camera_query.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else {
        return;
    };

    let mut best_idx = None;
    let mut best_dist = NODE_CLICK_RADIUS;

    for idx in graph.allocatable_nodes() {
        let node_pos = graph.nodes[idx].position;
        let dist = world_pos.distance(node_pos);
        if dist < best_dist {
            best_dist = dist;
            best_idx = Some(idx);
        }
    }

    if let Some(idx) = best_idx {
        allocate_events.write(AllocateNodeRequest { node_index: idx });
    }
}

pub fn update_node_visuals(
    graph: Option<Res<SkillGraph>>,
    node_query: Query<(&SkillTreeNode, &MeshMaterial2d<ColorMaterial>)>,
    edge_query: Query<(&SkillTreeEdge, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(graph) = graph else { return };
    if !graph.is_changed() {
        return;
    }

    for (node, mat_handle) in &node_query {
        let color = node_color(&graph, node.graph_index);
        if let Some(mat) = materials.get_mut(mat_handle.id()) {
            mat.color = color;
        }
    }

    for (edge, mat_handle) in &edge_query {
        let alpha = edge_alpha(&graph, edge.edge_index);
        if let Some(mat) = materials.get_mut(mat_handle.id()) {
            mat.color = Color::srgba(0.5, 0.5, 0.6, alpha);
        }
    }
}

pub fn skill_tree_hover(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SkillTreeCamera>>,
    graph: Option<Res<SkillGraph>>,
    pool: Option<Res<PassiveNodePool>>,
    display: Option<Res<StatDisplayRegistry>>,
    existing_tooltip: Query<Entity, With<SkillTreeTooltip>>,
    mut last_hovered: Local<Option<usize>>,
) {
    for entity in &existing_tooltip {
        commands.entity(entity).despawn();
    }

    let Some(graph) = graph else {
        *last_hovered = None;
        return;
    };
    let Some(pool) = pool else {
        *last_hovered = None;
        return;
    };
    let Some(display) = display else {
        *last_hovered = None;
        return;
    };

    let Ok(window) = windows.single() else {
        *last_hovered = None;
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        *last_hovered = None;
        return;
    };
    let Ok((camera, cam_gt)) = camera_query.single() else {
        *last_hovered = None;
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_gt, cursor_pos) else {
        *last_hovered = None;
        return;
    };

    let mut best_idx = None;
    let mut best_dist = NODE_CLICK_RADIUS;

    for (idx, node) in graph.nodes.iter().enumerate() {
        let dist = world_pos.distance(node.position);
        if dist < best_dist {
            best_dist = dist;
            best_idx = Some(idx);
        }
    }

    *last_hovered = best_idx;

    if let Some(idx) = best_idx {
        spawn_tooltip(&mut commands, &graph, &pool, &display, idx, cursor_pos);
    }
}

fn spawn_tooltip(
    commands: &mut Commands,
    graph: &SkillGraph,
    pool: &PassiveNodePool,
    display: &StatDisplayRegistry,
    idx: usize,
    cursor_pos: Vec2,
) {
    let node = &graph.nodes[idx];
    if node.is_start() {
        return;
    }

    let def = &pool.defs[node.def_index];

    let left = (cursor_pos.x + 16.0).max(0.0);
    let top = (cursor_pos.y - 16.0).max(0.0);

    let tooltip = commands
        .spawn((
            SkillTreeTooltip,
            GlobalZIndex(200),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(12.0)),
                min_width: Val::Px(200.0),
                ..default()
            },
            BackgroundColor(TOOLTIP_BG),
        ))
        .id();

    let status = if node.allocated {
        " (Learned)"
    } else if graph.is_allocatable(idx) {
        " (Available)"
    } else {
        ""
    };

    let header = commands
        .spawn((
            Text(format!("{}{}", def.name, status)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(GOLD_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(tooltip).add_child(header);

    for &(stat, value) in &node.rolled_values {
        let formats = display.get_format(&[stat]);
        if formats.is_empty() {
            continue;
        }
        let row = StatLineBuilder::spawn_line(
            commands,
            &formats[0],
            StatRenderMode::Fixed { values: &[value] },
            16.0,
        );
        commands.entity(row).insert(Node {
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        });
        commands.entity(tooltip).add_child(row);
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

pub fn cleanup_skill_tree_view(
    mut commands: Commands,
    camera_query: Query<Entity, With<SkillTreeCamera>>,
    world_query: Query<Entity, With<SkillTreeWorld>>,
) {
    for entity in &camera_query {
        commands.entity(entity).despawn();
    }
    for entity in &world_query {
        commands.entity(entity).despawn();
    }
}
