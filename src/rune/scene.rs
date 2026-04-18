use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::balance::GameBalance;
use crate::coord;
use crate::palette;
use crate::run::PlayerMoney;
use crate::schedule::ShopSet;
use crate::wave::WavePhase;

use super::content::{write_pattern_contains, write_pattern_coords, write_targets};
use super::cost::RuneCosts;
use super::data::{
    can_place, Dragging, GridCellView, GridHighlights, JokerSlotView, JokerSlots, Rune,
    RuneGrid, RuneSource, RuneView, ShopOffer, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
use super::hex::HexCoord;
use super::shop_gen::roll_shop_offer;

pub const CELL_SIDE_WORLD: f32 = 100.0;
pub const BALL_RADIUS: f32 = 60.0;
pub const BALL_ELEVATION: f32 = BALL_RADIUS;
pub const DRAG_LIFT: f32 = BALL_RADIUS * 1.0;

const CELL_RING_INNER: f32 = 65.0;
const CELL_RING_OUTER: f32 = 70.0;
const JOKER_RING_INNER: f32 = 54.0;
const JOKER_RING_OUTER: f32 = 64.0;
const HIGHLIGHT_RING_INNER: f32 = BALL_RADIUS + 4.0;
const HIGHLIGHT_RING_OUTER: f32 = BALL_RADIUS + 12.0;
const PATTERN_RING_INNER: f32 = CELL_RING_OUTER + 2.0;
const PATTERN_RING_OUTER: f32 = CELL_RING_OUTER + 10.0;
const SHOP_BALL_X: f32 = 640.0;
const SHOP_BALL_Z_GAP: f32 = 140.0;
const PRICE_LABEL_Y_OFFSET: f32 = BALL_RADIUS + 28.0;

const GROUND_Y: f32 = 0.02;
const HIGHLIGHT_Y: f32 = 0.04;

const JOKER_HEX_COORDS: [(i32, i32); JOKER_SLOTS] = [
    (4, -2),
    (2, -4),
    (-2, -2),
    (-4, 2),
    (-2, 4),
    (2, 2),
];

#[derive(Resource)]
struct SceneMeshes {
    ball: Handle<Mesh>,
    cell_ring: Handle<Mesh>,
    joker_ring: Handle<Mesh>,
    highlight_ring: Handle<Mesh>,
    pattern_ring: Handle<Mesh>,
    shadow: Handle<Mesh>,
}

#[derive(Resource, Default)]
struct BallMaterials(HashMap<u32, Handle<StandardMaterial>>);

#[derive(Resource)]
struct GroundMaterials {
    locked: Handle<StandardMaterial>,
    unlocked: Handle<StandardMaterial>,
    joker: Handle<StandardMaterial>,
    write_target: Handle<StandardMaterial>,
    write_source: Handle<StandardMaterial>,
    pattern: Handle<StandardMaterial>,
    shadow: Handle<StandardMaterial>,
}

#[derive(Component)]
struct HighlightDecal;

#[derive(Component)]
struct BallShadow;

#[derive(Component)]
pub struct ShopPriceLabel {
    pub index: usize,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, setup_scene_assets)
        .add_systems(
            OnEnter(WavePhase::Shop),
            (reset_reroll_cost, fill_shop_offer_system, spawn_shop_scene).chain(),
        )
        .add_systems(OnExit(WavePhase::Shop), restore_dragged_on_exit)
        .add_systems(
            Update,
            (
                start_drag.in_set(ShopSet::Input),
                finish_drag.in_set(ShopSet::Input),
                reconcile_rune_entities.in_set(ShopSet::Display),
                sync_cell_lock_visuals.in_set(ShopSet::Display),
                update_highlights.in_set(ShopSet::Display),
                apply_highlights.in_set(ShopSet::Display),
                update_shop_price_labels.in_set(ShopSet::Display),
            )
                .run_if(in_state(WavePhase::Shop)),
        )
        .add_systems(
            PostUpdate,
            (follow_cursor, sync_ball_shadows)
                .chain()
                .run_if(in_state(WavePhase::Shop)),
        );
}

fn setup_scene_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(SceneMeshes {
        ball: meshes.add(Sphere::new(BALL_RADIUS)),
        cell_ring: meshes.add(Annulus::new(CELL_RING_INNER, CELL_RING_OUTER)),
        joker_ring: meshes.add(Annulus::new(JOKER_RING_INNER, JOKER_RING_OUTER)),
        highlight_ring: meshes.add(Annulus::new(HIGHLIGHT_RING_INNER, HIGHLIGHT_RING_OUTER)),
        pattern_ring: meshes.add(Annulus::new(PATTERN_RING_INNER, PATTERN_RING_OUTER)),
        shadow: meshes.add(Circle::new(BALL_RADIUS * 0.95)),
    });
    let shadow_mat = StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.35),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    commands.insert_resource(GroundMaterials {
        locked: materials.add(unlit_material(palette::color("ui_cell_locked_bg"))),
        unlocked: materials.add(unlit_material(palette::color("ui_cell_unlocked_bg"))),
        joker: materials.add(unlit_material(palette::color("ui_joker_slot_edge"))),
        write_target: materials.add(unlit_material(palette::color("ui_rune_write_target"))),
        write_source: materials.add(unlit_material(palette::color("ui_rune_write_source"))),
        pattern: materials.add(unlit_material(palette::color("ui_rune_write_target"))),
        shadow: materials.add(shadow_mat),
    });
    commands.insert_resource(BallMaterials::default());
}

fn unlit_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    }
}

fn reset_reroll_cost(
    balance: Res<GameBalance>,
    mut reroll: ResMut<super::data::RerollState>,
) {
    reroll.cost = balance.runes.reroll_base_cost;
}

fn fill_shop_offer_system(
    mut offer: ResMut<ShopOffer>,
    grid: Res<RuneGrid>,
    balance: Res<GameBalance>,
    costs: Res<RuneCosts>,
) {
    if offer.stubs.iter().any(|s| s.is_some()) {
        return;
    }
    roll_shop_offer(&mut offer, &grid, &balance, &costs);
}

fn spawn_shop_scene(mut commands: Commands, meshes: Res<SceneMeshes>, ground: Res<GroundMaterials>) {
    for coord in HexCoord::all_within_radius(GRID_RADIUS) {
        commands.spawn((
            Name::new("RuneCell"),
            GridCellView { coord },
            Mesh3d(meshes.cell_ring.clone()),
            MeshMaterial3d(ground.locked.clone()),
            cell_transform(cell_world_pos(coord), GROUND_Y),
            DespawnOnExit(WavePhase::Shop),
        ));
    }

    for idx in 0..JOKER_SLOTS {
        commands.spawn((
            Name::new("JokerSlot"),
            JokerSlotView,
            Mesh3d(meshes.joker_ring.clone()),
            MeshMaterial3d(ground.joker.clone()),
            cell_transform(joker_world_pos(idx), GROUND_Y),
            DespawnOnExit(WavePhase::Shop),
        ));
    }
}

fn cell_transform(pos: Vec3, y: f32) -> Transform {
    Transform::from_translation(Vec3::new(pos.x, y, pos.z))
        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
}

pub fn cell_world_pos(coord: HexCoord) -> Vec3 {
    coord::ground_pos(coord.to_pixel(CELL_SIDE_WORLD))
}

pub fn shop_world_pos(idx: usize) -> Vec3 {
    let total = SHOP_SLOTS as f32;
    let z = (idx as f32 - (total - 1.0) * 0.5) * SHOP_BALL_Z_GAP;
    Vec3::new(SHOP_BALL_X, 0.0, z)
}

pub fn joker_world_pos(idx: usize) -> Vec3 {
    let (q, r) = JOKER_HEX_COORDS[idx];
    cell_world_pos(HexCoord::new(q, r))
}

pub fn home_world_pos(source: RuneSource) -> Vec3 {
    match source {
        RuneSource::Shop(idx) => shop_world_pos(idx),
        RuneSource::Grid(coord) => cell_world_pos(coord),
        RuneSource::Joker(idx) => joker_world_pos(idx),
    }
}

fn ball_material(
    rune: Rune,
    cache: &mut BallMaterials,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let key = color_key(rune.color);
    cache
        .0
        .entry(key)
        .or_insert_with(|| materials.add(unlit_material(rune.color)))
        .clone()
}

fn color_key(color: Color) -> u32 {
    let srgba = color.to_srgba();
    let r = (srgba.red.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (srgba.green.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (srgba.blue.clamp(0.0, 1.0) * 255.0) as u32;
    let a = (srgba.alpha.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 24) | (g << 16) | (b << 8) | a
}

fn spawn_rune_entity(
    commands: &mut Commands,
    rune: Rune,
    source: RuneSource,
    meshes: &SceneMeshes,
    cache: &mut BallMaterials,
    materials: &mut Assets<StandardMaterial>,
    ground_shadow: &Handle<StandardMaterial>,
) {
    let pos = home_world_pos(source);
    let material = ball_material(rune, cache, materials);
    let mut entity = commands.spawn((
        Name::new("RuneBall"),
        RuneView { source, rune_id: rune.id },
        Mesh3d(meshes.ball.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(pos.x, BALL_ELEVATION, pos.z)),
        DespawnOnExit(WavePhase::Shop),
    ));
    entity.with_children(|p| {
        p.spawn((
            Name::new("BallShadow"),
            BallShadow,
            Mesh3d(meshes.shadow.clone()),
            MeshMaterial3d(ground_shadow.clone()),
            Transform::from_translation(Vec3::new(0.0, GROUND_Y - BALL_ELEVATION, 0.0))
                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        ));
    });
    if matches!(source, RuneSource::Shop(_)) {
        let idx = match source {
            RuneSource::Shop(i) => i,
            _ => 0,
        };
        entity.with_children(|p| {
            p.spawn((
                ShopPriceLabel { index: idx },
                Text2d::new(format!("{}", rune.cost)),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(palette::color("ui_text_money")),
                Transform::from_translation(Vec3::new(0.0, PRICE_LABEL_Y_OFFSET, 0.0)),
            ));
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn reconcile_rune_entities(
    mut commands: Commands,
    shop: Res<ShopOffer>,
    grid: Res<RuneGrid>,
    jokers: Res<JokerSlots>,
    meshes: Res<SceneMeshes>,
    ground: Res<GroundMaterials>,
    mut cache: ResMut<BallMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<(Entity, &RuneView), Without<Dragging>>,
) {
    let mut by_source: HashMap<RuneSource, (Entity, u32)> = HashMap::new();
    for (entity, view) in &existing {
        if let Some((duplicate, _)) = by_source.insert(view.source, (entity, view.rune_id)) {
            commands.entity(duplicate).despawn();
        }
    }

    let shadow_handle = ground.shadow.clone();
    let mut reconcile = |commands: &mut Commands,
                         cache: &mut BallMaterials,
                         materials: &mut Assets<StandardMaterial>,
                         src: RuneSource,
                         rune: Rune| {
        match by_source.remove(&src) {
            Some((_, id)) if id == rune.id => {}
            Some((entity, _)) => {
                commands.entity(entity).despawn();
                spawn_rune_entity(commands, rune, src, &meshes, cache, materials, &shadow_handle);
            }
            None => {
                spawn_rune_entity(commands, rune, src, &meshes, cache, materials, &shadow_handle);
            }
        }
    };

    for (idx, slot) in shop.stubs.iter().enumerate() {
        if let Some(rune) = slot {
            reconcile(
                &mut commands,
                &mut cache,
                &mut materials,
                RuneSource::Shop(idx),
                *rune,
            );
        }
    }
    for (coord, rune) in grid.cells.iter() {
        reconcile(
            &mut commands,
            &mut cache,
            &mut materials,
            RuneSource::Grid(*coord),
            *rune,
        );
    }
    for (idx, slot) in jokers.stubs.iter().enumerate() {
        if let Some(rune) = slot {
            reconcile(
                &mut commands,
                &mut cache,
                &mut materials,
                RuneSource::Joker(idx),
                *rune,
            );
        }
    }

    for (_, (entity, _)) in by_source {
        commands.entity(entity).despawn();
    }
}

fn sync_cell_lock_visuals(
    grid: Res<RuneGrid>,
    ground: Res<GroundMaterials>,
    mut cells: Query<(&GridCellView, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for (cell, mut mat) in &mut cells {
        let target = if grid.is_unlocked(cell.coord) {
            &ground.unlocked
        } else {
            &ground.locked
        };
        if mat.0.id() != target.id() {
            mat.0 = target.clone();
        }
    }
}

fn update_shop_price_labels(
    offer: Res<ShopOffer>,
    mut labels: Query<(&ShopPriceLabel, &mut Text2d)>,
) {
    if !offer.is_changed() {
        return;
    }
    for (label, mut text) in &mut labels {
        if let Some(Some(rune)) = offer.stubs.get(label.index) {
            text.0 = format!("{}", rune.cost);
        }
    }
}

fn cursor_world(
    window_q: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) -> Option<Vec3> {
    let window = window_q.single().ok()?;
    let (camera, transform) = camera_q.single().ok()?;
    coord::cursor_ground_pos(window, camera, transform)
}

pub fn find_drop_target_world(
    cursor_world: Vec3,
    is_joker: bool,
    grid: &RuneGrid,
) -> Option<RuneSource> {
    let cursor_2d = coord::to_2d(cursor_world);
    if !is_joker {
        let coord = HexCoord::from_pixel(cursor_2d, CELL_SIDE_WORLD);
        if coord.ring_radius() > GRID_RADIUS {
            return None;
        }
        if !grid.is_unlocked(coord) {
            return None;
        }
        return Some(RuneSource::Grid(coord));
    }
    for idx in 0..JOKER_SLOTS {
        let pos = joker_world_pos(idx);
        if (Vec2::new(pos.x, pos.z) - Vec2::new(cursor_world.x, cursor_world.z)).length()
            <= JOKER_RING_OUTER
        {
            return Some(RuneSource::Joker(idx));
        }
    }
    None
}

fn start_drag(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
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
    let Ok(window) = window_q.single() else { return };
    let Ok((camera, transform)) = camera_q.single() else { return };
    let Some(pick) = coord::cursor_plane_pos(window, camera, transform, BALL_ELEVATION)
    else {
        return;
    };
    let Some(ground) = coord::cursor_ground_pos(window, camera, transform) else {
        return;
    };
    let pick_radius_sq = BALL_RADIUS * BALL_RADIUS;
    for (entity, view) in &runes {
        let home = home_world_pos(view.source);
        let dx = home.x - pick.x;
        let dz = home.z - pick.z;
        if dx * dx + dz * dz > pick_radius_sq {
            continue;
        }
        if let RuneSource::Shop(_) = view.source {
            let cost = peek_rune(view.source, &shop, &grid, &jokers)
                .map(|r| r.cost)
                .unwrap_or(0);
            if !money.can_afford(cost) {
                return;
            }
        }
        let Some(rune) = take_rune(view.source, &mut shop, &mut grid, &mut jokers) else {
            return;
        };
        let grab_offset = Vec3::new(home.x - ground.x, 0.0, home.z - ground.z);
        commands.entity(entity).insert(Dragging {
            rune,
            from: view.source,
            grab_offset,
        });
        return;
    }
}

const FOLLOW_SMOOTHING: f32 = 28.0;

fn follow_cursor(
    time: Res<Time>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut runes: Query<(&RuneView, Option<&Dragging>, &mut Transform)>,
) {
    let cursor = cursor_world(&window_q, &camera_q);
    let dt = time.delta_secs();
    let alpha = (1.0 - (-FOLLOW_SMOOTHING * dt).exp()).clamp(0.0, 1.0);
    for (view, dragging, mut transform) in &mut runes {
        let target = match (dragging, cursor) {
            (Some(drag), Some(c)) => Vec3::new(c.x + drag.grab_offset.x, DRAG_LIFT, c.z + drag.grab_offset.z),
            _ => {
                let h = home_world_pos(view.source);
                Vec3::new(h.x, BALL_ELEVATION, h.z)
            }
        };
        transform.translation = transform.translation.lerp(target, alpha);
    }
}

fn sync_ball_shadows(
    parents: Query<&Transform, (With<RuneView>, Without<BallShadow>)>,
    mut shadows: Query<(&ChildOf, &mut Transform), With<BallShadow>>,
) {
    for (child_of, mut t) in &mut shadows {
        let Ok(parent) = parents.get(child_of.0) else { continue };
        t.translation.y = GROUND_Y - parent.translation.y;
    }
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

#[allow(clippy::too_many_arguments)]
fn finish_drag(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut money: ResMut<PlayerMoney>,
    mut shop: ResMut<ShopOffer>,
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    mut views: Query<(Entity, &mut RuneView, Option<&Dragging>)>,
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
    let cursor = cursor_world(&window_q, &camera_q);
    let target = cursor.and_then(|c| {
        let center = Vec3::new(c.x + drag.grab_offset.x, 0.0, c.z + drag.grab_offset.z);
        find_drop_target_world(center, drag.rune.is_joker(), &grid)
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

#[allow(clippy::too_many_arguments)]
fn update_highlights(
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    grid: Res<RuneGrid>,
    dragging: Query<(Entity, &Dragging)>,
    views: Query<(Entity, &RuneView), Without<Dragging>>,
    mut highlights: ResMut<GridHighlights>,
) {
    highlights.center_entity = None;
    highlights.center_pos = None;
    highlights.write_targets.clear();
    highlights.write_sources.clear();
    highlights.pattern_cells.clear();

    let Some(cursor) = cursor_world(&window_q, &camera_q) else {
        return;
    };

    if let Some((drag_entity, drag)) = dragging.iter().next() {
        let center = Vec3::new(cursor.x + drag.grab_offset.x, 0.0, cursor.z + drag.grab_offset.z);
        let Some(target) = find_drop_target_world(center, drag.rune.is_joker(), &grid) else {
            return;
        };
        let RuneSource::Grid(c) = target else { return };
        highlights.center_entity = Some(drag_entity);
        highlights.center_pos = Some(cell_world_pos(c));

        if let Some(write) = drag.rune.kind.and_then(|k| k.def().write) {
            for t in write_targets(&write, c, &grid) {
                highlights.write_targets.insert(t);
            }
            for p in write_pattern_coords(&write, c) {
                highlights.pattern_cells.insert(p);
            }
        }
        for (src_coord, src_rune) in grid.cells.iter() {
            if *src_coord == c {
                continue;
            }
            let Some(src_kind) = src_rune.kind else { continue };
            let Some(write) = src_kind.def().write else { continue };
            if write_pattern_contains(&write, *src_coord, c) {
                highlights.write_sources.insert(*src_coord);
            }
        }
        return;
    }

    let hover_coord = {
        let c = HexCoord::from_pixel(coord::to_2d(cursor), CELL_SIDE_WORLD);
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
    highlights.center_pos = Some(cell_world_pos(coord));

    if let Some(write) = kind.def().write {
        for t in write_targets(&write, coord, &grid) {
            highlights.write_targets.insert(t);
        }
        for p in write_pattern_coords(&write, coord) {
            highlights.pattern_cells.insert(p);
        }
    }
    for (src_coord, src_rune) in grid.cells.iter() {
        if *src_coord == coord {
            continue;
        }
        let Some(src_kind) = src_rune.kind else { continue };
        let Some(write) = src_kind.def().write else { continue };
        if write_pattern_contains(&write, *src_coord, coord) {
            highlights.write_sources.insert(*src_coord);
        }
    }
}

fn apply_highlights(
    mut commands: Commands,
    highlights: Res<GridHighlights>,
    meshes: Res<SceneMeshes>,
    ground: Res<GroundMaterials>,
    decals: Query<Entity, With<HighlightDecal>>,
) {
    for entity in &decals {
        commands.entity(entity).despawn();
    }

    for coord in highlights.pattern_cells.iter() {
        let pos = cell_world_pos(*coord);
        commands.spawn((
            Name::new("PatternRing"),
            HighlightDecal,
            Mesh3d(meshes.pattern_ring.clone()),
            MeshMaterial3d(ground.pattern.clone()),
            cell_transform(pos, HIGHLIGHT_Y),
            DespawnOnExit(WavePhase::Shop),
        ));
    }

    for coord in highlights.write_targets.iter() {
        let pos = cell_world_pos(*coord);
        commands.spawn((
            Name::new("WriteTargetRing"),
            HighlightDecal,
            Mesh3d(meshes.highlight_ring.clone()),
            MeshMaterial3d(ground.write_target.clone()),
            cell_transform(pos, HIGHLIGHT_Y),
            DespawnOnExit(WavePhase::Shop),
        ));
    }
    for coord in highlights.write_sources.iter() {
        let pos = cell_world_pos(*coord);
        commands.spawn((
            Name::new("WriteSourceRing"),
            HighlightDecal,
            Mesh3d(meshes.highlight_ring.clone()),
            MeshMaterial3d(ground.write_source.clone()),
            cell_transform(pos, HIGHLIGHT_Y),
            DespawnOnExit(WavePhase::Shop),
        ));
    }
}

fn restore_dragged_on_exit(
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
