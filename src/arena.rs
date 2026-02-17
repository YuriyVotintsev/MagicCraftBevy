use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::camera::ScalingMode;
use rand::Rng;

use crate::balance::{ArenaBalance, GameBalance};
use crate::GameState;
use crate::Faction;
use crate::blueprints::{BlueprintRegistry, spawn_blueprint_entity};
use crate::blueprints::components::common::health::Health;
use crate::physics::{GameLayer, Wall};
use crate::schedule::GameSet;
use crate::wave::{WaveEnemy, WavePhase, WaveState};

#[cfg(not(feature = "headless"))]
pub const WINDOW_WIDTH: f32 = 1920.0;
#[cfg(not(feature = "headless"))]
pub const WINDOW_HEIGHT: f32 = 1080.0;

const MARKER_SIZE: f32 = 30.0;

#[derive(Component)]
struct SpawnMarker {
    timer: Timer,
    blueprint_name: String,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::MainMenu),
                spawn_arena.run_if(not(any_with_component::<Wall>)),
            )
            .add_systems(OnEnter(GameState::Playing), cleanup_spawn_markers)
            .add_systems(OnExit(WavePhase::Combat), cleanup_spawn_markers)
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

fn setup_camera(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::BLACK));
    commands.spawn((
        Name::new("MainCamera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn spawn_arena(mut commands: Commands, asset_server: Res<AssetServer>, balance: Res<GameBalance>) {
    let arena = &balance.arena;

    commands.spawn((
        Name::new("Background"),
        Sprite {
            image: asset_server.load("images/sprites/Background.png"),
            custom_size: Some(Vec2::new(arena.width, arena.height)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    let half_w = arena.half_w;
    let half_h = arena.half_h;
    let r = arena.corner_radius;
    let segments = 8_u32;

    let ix = half_w - r;
    let iy = half_h - r;
    let corners = [
        (ix, iy, 0.0_f32, 90.0_f32),
        (-ix, iy, 90.0, 180.0),
        (-ix, -iy, 180.0, 270.0),
        (ix, -iy, 270.0, 360.0),
    ];

    let mut vertices = Vec::new();
    for (cx, cy, start, end) in corners {
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = (start + t * (end - start)).to_radians();
            vertices.push(Vec2::new(cx + r * angle.cos(), cy + r * angle.sin()));
        }
    }

    let n = vertices.len() as u32;
    let indices: Vec<[u32; 2]> = (0..n).map(|i| [i, (i + 1) % n]).collect();

    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    commands.spawn((
        Name::new("ArenaWall"),
        Wall,
        Transform::default(),
        Collider::polyline(vertices, Some(indices)),
        CollisionMargin(5.0),
        RigidBody::Static,
        wall_layers,
    ));
}

fn camera_follow(
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<crate::player::Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn spawn_markers(
    mut commands: Commands,
    wave_state: Res<WaveState>,
    markers_query: Query<(), With<SpawnMarker>>,
    enemies_query: Query<&Faction, With<Health>>,
    balance: Res<GameBalance>,
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
            Sprite {
                color: Color::srgb(1.0, 0.9, 0.2),
                custom_size: Some(Vec2::splat(MARKER_SIZE)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.5),
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
    for (marker_entity, mut marker, _transform) in markers_query.iter_mut() {
        if marker.timer.tick(time.delta()).just_finished() {
            if let Some(blueprint_id) = blueprint_registry.get_id(&marker.blueprint_name) {
                spawn_blueprint_entity(&mut commands, marker_entity, Faction::Enemy, blueprint_id, true);
                wave_state.spawned_count += 1;
            }

            commands.entity(marker_entity).remove::<(Sprite, SpawnMarker)>();
            commands.entity(marker_entity).insert((WaveEnemy, DespawnOnExit(WavePhase::Combat)));
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
