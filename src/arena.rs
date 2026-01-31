use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::GameState;
use crate::abilities::{AbilityRegistry, NodeRegistry};
use crate::fsm::{spawn_mob, BehaviourRegistry, MobRegistry, TransitionRegistry};
use crate::physics::{GameLayer, Wall};
use crate::stats::{StatCalculators, StatRegistry};
use crate::wave::{WaveEnemy, WavePhase, WaveState};

#[cfg(not(feature = "headless"))]
pub const WINDOW_WIDTH: f32 = 1280.0;
#[cfg(not(feature = "headless"))]
pub const WINDOW_HEIGHT: f32 = 720.0;
pub const ARENA_WIDTH: f32 = 1920.0;
pub const ARENA_HEIGHT: f32 = 1080.0;
pub const BORDER_THICKNESS: f32 = 10.0;

const SLIME_SPAWN_INTERVAL: f32 = 1.5;
const SLIME_SIZE: f32 = 30.0;

#[derive(Resource)]
struct SlimeSpawnTimer(Timer);

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SlimeSpawnTimer(Timer::from_seconds(
            SLIME_SPAWN_INTERVAL,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (setup_camera, spawn_arena))
        .add_systems(OnEnter(GameState::Playing), reset_spawn_timer)
        .add_systems(
            Update,
            spawn_enemies.run_if(in_state(WavePhase::Combat)),
        )
        .add_systems(
            PostUpdate,
            camera_follow.run_if(in_state(GameState::Playing)),
        );
    }
}

fn reset_spawn_timer(mut timer: ResMut<SlimeSpawnTimer>) {
    timer.0.reset();
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

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<SlimeSpawnTimer>,
    mob_registry: Res<MobRegistry>,
    stat_registry: Res<StatRegistry>,
    calculators: Res<StatCalculators>,
    ability_registry: Res<AbilityRegistry>,
    node_registry: Res<NodeRegistry>,
    behaviour_registry: Res<BehaviourRegistry>,
    transition_registry: Res<TransitionRegistry>,
    mut wave_state: ResMut<WaveState>,
) {
    if wave_state.spawned_count >= wave_state.target_count {
        return;
    }

    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::rng();
        let half_width = ARENA_WIDTH / 2.0 - SLIME_SIZE / 2.0;
        let half_height = ARENA_HEIGHT / 2.0 - SLIME_SIZE / 2.0;

        let x = rng.random_range(-half_width..half_width);
        let y = rng.random_range(-half_height..half_height);

        let mob_name = if rng.random_bool(0.5) { "slime" } else { "archer" };

        if let Some(entity) = spawn_mob(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mob_registry,
            &stat_registry,
            &calculators,
            &ability_registry,
            &node_registry,
            &behaviour_registry,
            &transition_registry,
            mob_name,
            Vec3::new(x, y, 1.0),
        ) {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(WaveEnemy);
            }
            wave_state.spawned_count += 1;
        }
    }
}
