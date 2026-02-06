use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::GameState;
use crate::Faction;
use crate::abilities::{AbilityRegistry, attach_ability};
use crate::abilities::components::health::Health;
use crate::physics::{GameLayer, Wall};
use crate::wave::{WaveEnemy, WavePhase, WaveState};

#[cfg(not(feature = "headless"))]
pub const WINDOW_WIDTH: f32 = 1280.0;
#[cfg(not(feature = "headless"))]
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const ARENA_WIDTH: f32 = 1920.0;
pub const ARENA_HEIGHT: f32 = 1080.0;
pub const BORDER_THICKNESS: f32 = 10.0;

const MARKER_SIZE: f32 = 30.0;
const MARKER_DURATION: f32 = 2.0;

#[derive(Component)]
struct SpawnMarker {
    timer: Timer,
    ability_name: String,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_camera, spawn_arena))
            .add_systems(OnEnter(GameState::Playing), cleanup_spawn_markers)
            .add_systems(OnExit(WavePhase::Combat), cleanup_spawn_markers)
            .add_systems(
                Update,
                (spawn_markers, process_spawn_markers, tag_wave_enemies)
                    .chain()
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
    commands.spawn((Name::new("MainCamera"), Camera2d));
}

fn spawn_arena(mut commands: Commands) {
    let half_width = ARENA_WIDTH / 2.0;
    let half_height = ARENA_HEIGHT / 2.0;
    let border_color = Color::srgb(0.8, 0.8, 0.8);

    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    commands.spawn((
        Name::new("Wall_Top"),
        Wall,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                ARENA_WIDTH + BORDER_THICKNESS * 2.0,
                BORDER_THICKNESS,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, half_height + BORDER_THICKNESS / 2.0, 0.0),
        Collider::rectangle(ARENA_WIDTH + BORDER_THICKNESS * 2.0, BORDER_THICKNESS),
        RigidBody::Static,
        wall_layers,
    ));

    commands.spawn((
        Name::new("Wall_Bottom"),
        Wall,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                ARENA_WIDTH + BORDER_THICKNESS * 2.0,
                BORDER_THICKNESS,
            )),
            ..default()
        },
        Transform::from_xyz(0.0, -half_height - BORDER_THICKNESS / 2.0, 0.0),
        Collider::rectangle(ARENA_WIDTH + BORDER_THICKNESS * 2.0, BORDER_THICKNESS),
        RigidBody::Static,
        wall_layers,
    ));

    commands.spawn((
        Name::new("Wall_Left"),
        Wall,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                BORDER_THICKNESS,
                ARENA_HEIGHT + BORDER_THICKNESS * 2.0,
            )),
            ..default()
        },
        Transform::from_xyz(-half_width - BORDER_THICKNESS / 2.0, 0.0, 0.0),
        Collider::rectangle(BORDER_THICKNESS, ARENA_HEIGHT + BORDER_THICKNESS * 2.0),
        RigidBody::Static,
        wall_layers,
    ));

    commands.spawn((
        Name::new("Wall_Right"),
        Wall,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(
                BORDER_THICKNESS,
                ARENA_HEIGHT + BORDER_THICKNESS * 2.0,
            )),
            ..default()
        },
        Transform::from_xyz(half_width + BORDER_THICKNESS / 2.0, 0.0, 0.0),
        Collider::rectangle(BORDER_THICKNESS, ARENA_HEIGHT + BORDER_THICKNESS * 2.0),
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
) {
    let alive_or_spawning = wave_state.spawned_count.saturating_sub(wave_state.killed_count);
    let active_markers = markers_query.iter().count() as u32;

    if alive_or_spawning > WaveState::spawn_threshold() {
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
        .saturating_sub(alive_or_spawning)
        .saturating_sub(active_markers);
    let to_spawn = can_spawn.min(remaining_to_spawn);

    if to_spawn == 0 {
        return;
    }

    let mut rng = rand::rng();
    let half_width = ARENA_WIDTH / 2.0 - MARKER_SIZE / 2.0;
    let half_height = ARENA_HEIGHT / 2.0 - MARKER_SIZE / 2.0;

    for _ in 0..to_spawn {
        let x = rng.random_range(-half_width..half_width);
        let y = rng.random_range(-half_height..half_height);
        let ability_name = if rng.random_bool(0.5) {
            "slime"
        } else {
            "archer"
        };

        commands.spawn((
            Name::new("SpawnMarker"),
            SpawnMarker {
                timer: Timer::from_seconds(MARKER_DURATION, TimerMode::Once),
                ability_name: ability_name.to_string(),
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
    ability_registry: Res<AbilityRegistry>,
    mut wave_state: ResMut<WaveState>,
) {
    for (marker_entity, mut marker, _transform) in markers_query.iter_mut() {
        if marker.timer.tick(time.delta()).just_finished() {
            if let Some(ability_id) = ability_registry.get_id(&marker.ability_name) {
                attach_ability(&mut commands, marker_entity, Faction::Enemy, ability_id, &ability_registry);
                wave_state.spawned_count += 1;
            }

            commands.entity(marker_entity).remove::<(Sprite, SpawnMarker)>();
            commands.entity(marker_entity).insert(DespawnOnExit(WavePhase::Combat));
        }
    }
}

fn tag_wave_enemies(
    mut commands: Commands,
    query: Query<Entity, (Added<Health>, With<Faction>)>,
    faction_query: Query<&Faction>,
) {
    for entity in &query {
        let Ok(faction) = faction_query.get(entity) else { continue };
        if *faction == Faction::Enemy {
            commands.entity(entity).insert((
                WaveEnemy,
                DespawnOnExit(WavePhase::Combat),
            ));
        }
    }
}
