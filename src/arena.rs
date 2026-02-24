use avian3d::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use rand::Rng;

use crate::balance::{ArenaBalance, GameBalance};
use crate::coords::{vec2_to_3d, COLLIDER_HALF_HEIGHT};
use crate::GameState;
use crate::Faction;
use crate::blueprints::{BlueprintRegistry, spawn_blueprint_entity};
use crate::blueprints::components::common::health::Health;
use crate::physics::{GameLayer, Wall};
use crate::schedule::GameSet;
use crate::wave::{CombatPhase, WaveEnemy, WavePhase, WaveState};

#[derive(Resource)]
pub struct RenderSettings {
    pub camera_angle: f32,
    pub sprite_tilt: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            camera_angle: 35.0,
            sprite_tilt: 45.0,
        }
    }
}

#[cfg(not(feature = "headless"))]
pub const WINDOW_WIDTH: f32 = 1920.0;
#[cfg(not(feature = "headless"))]
pub const WINDOW_HEIGHT: f32 = 1080.0;

#[derive(Resource, Default)]
pub struct CameraYaw(pub f32);

const MARKER_SIZE: f32 = 30.0;

#[derive(Component)]
struct SpawnMarker {
    timer: Timer,
    blueprint_name: String,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderSettings>()
            .init_resource::<CameraYaw>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::MainMenu),
                spawn_arena.run_if(not(any_with_component::<Wall>)),
            )
            .add_systems(OnEnter(GameState::Playing), cleanup_spawn_markers)
            .add_systems(OnEnter(CombatPhase::Running), lock_cursor)
            .add_systems(OnExit(CombatPhase::Running), unlock_cursor)
            .add_systems(
                Update,
                rotate_camera.run_if(in_state(CombatPhase::Running)),
            )
            .add_systems(
                Update,
                (spawn_markers, process_spawn_markers, tag_wave_enemies)
                    .chain()
                    .in_set(GameSet::Spawning)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                PostUpdate,
                camera_follow.run_if(in_state(GameState::Playing)),
            );
    }
}

fn cleanup_spawn_markers(mut commands: Commands, markers: Query<Entity, With<SpawnMarker>>) {
    for entity in markers.iter() {
        commands.entity(entity).despawn();
    }
}

const CAMERA_DISTANCE: f32 = 1200.0;
const CAMERA_LOOK_AHEAD: f32 = 200.0;

fn camera_offset(settings: &RenderSettings, yaw: f32) -> Vec3 {
    let angle_rad = settings.camera_angle.to_radians();
    let base = Vec3::new(0.0, CAMERA_DISTANCE * angle_rad.sin(), CAMERA_DISTANCE * angle_rad.cos());
    Quat::from_rotation_y(yaw) * base
}

fn setup_camera(mut commands: Commands, settings: Res<RenderSettings>) {
    commands.insert_resource(ClearColor(Color::BLACK));
    let offset = camera_offset(&settings, 0.0);
    commands.spawn((
        Name::new("MainCamera"),
        Camera3d::default(),
        Transform::from_translation(offset).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Camera2dOverlay"),
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
    commands.spawn((
        Name::new("AmbientLight"),
        AmbientLight {
            brightness: 1000.0,
            ..default()
        },
    ));
}

fn spawn_arena(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    balance: Res<GameBalance>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arena = &balance.arena;

    commands.spawn((
        Name::new("Background"),
        Mesh3d(meshes.add(Rectangle::new(arena.width, arena.height))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("images/sprites/Background.png")),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    let half_w = arena.half_w;
    let half_h = arena.half_h;
    let wall_thickness = 20.0;
    let wall_height = COLLIDER_HALF_HEIGHT * 2.0;
    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    let walls = [
        ("WallTop", Vec3::new(0.0, 0.0, -half_h - wall_thickness / 2.0), Vec3::new(half_w * 2.0 + wall_thickness * 2.0, wall_height, wall_thickness)),
        ("WallBottom", Vec3::new(0.0, 0.0, half_h + wall_thickness / 2.0), Vec3::new(half_w * 2.0 + wall_thickness * 2.0, wall_height, wall_thickness)),
        ("WallLeft", Vec3::new(-half_w - wall_thickness / 2.0, 0.0, 0.0), Vec3::new(wall_thickness, wall_height, half_h * 2.0)),
        ("WallRight", Vec3::new(half_w + wall_thickness / 2.0, 0.0, 0.0), Vec3::new(wall_thickness, wall_height, half_h * 2.0)),
    ];

    for (name, pos, size) in walls {
        commands.spawn((
            Name::new(name),
            Wall,
            Transform::from_translation(pos),
            Collider::cuboid(size.x, size.y, size.z),
            RigidBody::Static,
            wall_layers,
        ));
    }
}

fn rotate_camera(mut yaw: ResMut<CameraYaw>, motion: Res<AccumulatedMouseMotion>) {
    yaw.0 -= motion.delta.x * 0.003;
}

fn lock_cursor(mut cursor_query: Query<&mut CursorOptions, With<Window>>) {
    if let Ok(mut options) = cursor_query.single_mut() {
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }
}

fn unlock_cursor(mut cursor_query: Query<&mut CursorOptions, With<Window>>) {
    if let Ok(mut options) = cursor_query.single_mut() {
        options.grab_mode = CursorGrabMode::None;
        options.visible = true;
    }
}

fn camera_follow(
    settings: Res<RenderSettings>,
    yaw: Res<CameraYaw>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<crate::player::Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let forward_3d = Quat::from_rotation_y(yaw.0) * Vec3::new(0.0, 0.0, -1.0);
    let look_ahead = forward_3d * CAMERA_LOOK_AHEAD;
    let target = player_transform.translation + look_ahead;
    let offset = camera_offset(&settings, yaw.0);
    camera_transform.translation = target + offset;
    camera_transform.look_at(target, Vec3::Y);
}

fn spawn_markers(
    mut commands: Commands,
    wave_state: Res<WaveState>,
    markers_query: Query<(), With<SpawnMarker>>,
    enemies_query: Query<&Faction, With<Health>>,
    balance: Res<GameBalance>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let alive_enemies = enemies_query.iter().filter(|f| **f == Faction::Enemy).count() as u32;
    let active_markers = markers_query.iter().count() as u32;
    let total_on_arena = alive_enemies + active_markers;

    if total_on_arena > balance.wave.spawn_threshold {
        return;
    }

    let remaining_to_spawn = wave_state
        .target_count
        .saturating_sub(wave_state.spawned_count)
        .saturating_sub(active_markers);
    if remaining_to_spawn == 0 {
        return;
    }

    let can_spawn = wave_state
        .max_concurrent
        .saturating_sub(total_on_arena);
    let to_spawn = can_spawn.min(remaining_to_spawn);

    if to_spawn == 0 {
        return;
    }

    let arena = &balance.arena;
    let mut rng = rand::rng();
    let margin = MARKER_SIZE;
    let hw = arena.half_w - margin;
    let hh = arena.half_h - margin;

    for _ in 0..to_spawn {
        let (x, y) = loop {
            let x = rng.random_range(-hw..hw);
            let y = rng.random_range(-hh..hh);
            if is_inside_arena(Vec2::new(x, y), margin, arena) {
                break (x, y);
            }
        };
        let roll = rng.random_range(0..3);
        let blueprint_name = match roll {
            0 => "skeleton",
            1 => "archer",
            _ => "slime_giant",
        };

        commands.spawn((
            Name::new("SpawnMarker"),
            SpawnMarker {
                timer: Timer::from_seconds(balance.wave.marker_duration, TimerMode::Once),
                blueprint_name: blueprint_name.to_string(),
            },
            Mesh3d(meshes.add(Rectangle::new(MARKER_SIZE, MARKER_SIZE))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.9, 0.2),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(vec2_to_3d(Vec2::new(x, y)) + Vec3::Y * 0.1)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
    }
}

fn process_spawn_markers(
    mut commands: Commands,
    time: Res<Time>,
    mut markers_query: Query<(Entity, &mut SpawnMarker, &Transform)>,
    blueprint_registry: Res<BlueprintRegistry>,
    mut wave_state: ResMut<WaveState>,
) {
    for (marker_entity, mut marker, transform) in markers_query.iter_mut() {
        if marker.timer.tick(time.delta()).just_finished() {
            if let Some(blueprint_id) = blueprint_registry.get_id(&marker.blueprint_name) {
                spawn_blueprint_entity(&mut commands, marker_entity, Faction::Enemy, blueprint_id, true);
                wave_state.spawned_count += 1;
            }

            let pos = transform.translation;
            commands.entity(marker_entity).remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>, SpawnMarker)>();
            commands.entity(marker_entity).insert((
                Transform::from_translation(pos),
                WaveEnemy,
                DespawnOnExit(WavePhase::Combat),
            ));
        }
    }
}

fn is_inside_arena(pos: Vec2, margin: f32, arena: &ArenaBalance) -> bool {
    let hw = arena.half_w - margin;
    let hh = arena.half_h - margin;
    let r = (arena.corner_radius - margin).max(0.0);
    let px = pos.x.abs();
    let py = pos.y.abs();
    if px > hw || py > hh {
        return false;
    }
    let ix = hw - r;
    let iy = hh - r;
    if px > ix && py > iy {
        let dx = px - ix;
        let dy = py - iy;
        return dx * dx + dy * dy <= r * r;
    }
    true
}

fn tag_wave_enemies(
    mut commands: Commands,
    query: Query<Entity, (Added<Health>, With<Faction>, Without<WaveEnemy>)>,
    faction_query: Query<&Faction>,
) {
    for entity in &query {
        let Ok(faction) = faction_query.get(entity) else { continue };
        if *faction == Faction::Enemy {
            commands.entity(entity).insert(DespawnOnExit(WavePhase::Combat));
        }
    }
}
