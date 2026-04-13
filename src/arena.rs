use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::Tonemapping;
use rand::Rng;

use crate::actors::mobs::{MobKind, MobsBalance};
use crate::balance::{CurrentArenaSize, GameBalance};
use crate::actors::components::common::health::Health;
use crate::physics::{GameLayer, Wall};
use crate::run::RunState;
use crate::schedule::GameSet;
use crate::stats::Stat;
use crate::summoning::{SummoningCircle, SummoningCircleMaterial, SummoningCircleMesh};
use crate::wave::{WaveEnemy, WavePhase, WaveState};
use crate::Faction;
use crate::GameState;

pub const WINDOW_WIDTH: f32 = 1920.0;
pub const WINDOW_HEIGHT: f32 = 1080.0;

const ALL_ENEMY_TYPES: &[MobKind] = &[
    MobKind::SlimeSmall,
    MobKind::Jumper,
    MobKind::Tower,
    MobKind::Ghost,
    MobKind::Spinner,
];

#[derive(Resource)]
pub struct EnemySpawnPool {
    pub enabled: Vec<(MobKind, bool)>,
}

impl Default for EnemySpawnPool {
    fn default() -> Self {
        Self {
            enabled: ALL_ENEMY_TYPES.iter().map(|&k| (k, true)).collect(),
        }
    }
}

impl EnemySpawnPool {
    pub fn active_kinds(&self) -> Vec<MobKind> {
        self.enabled.iter().filter(|(_, on)| *on).map(|(k, _)| *k).collect()
    }
}

#[derive(Component)]
struct FloorParams {
    radius: f32,
}

#[derive(Component)]
enum WallSide { North, South, West, East }

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraAngle>()
            .init_resource::<EnemySpawnPool>()
            .init_resource::<CurrentArenaSize>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::MainMenu),
                spawn_arena.run_if(not(any_with_component::<Wall>)),
            )
            .add_systems(
                OnEnter(WavePhase::Combat),
                reset_arena_size,
            )
            .add_systems(
                Update,
                (update_arena_size, update_target_count, spawn_enemies, tag_wave_enemies)
                    .chain()
                    .in_set(GameSet::Spawning)
                    .run_if(in_state(WavePhase::Combat))
                    .run_if(not(resource_exists::<crate::run::PlayerDying>)),
            )
            .add_systems(
                PostUpdate,
                camera_follow.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, (update_walls, update_floor_mesh));
    }
}

const CAM_DISTANCE: f32 = 1000.0;

#[derive(Resource)]
pub struct CameraAngle {
    pub degrees: f32,
}

impl Default for CameraAngle {
    fn default() -> Self {
        Self { degrees: 55.0 }
    }
}

fn camera_offset(angle_degrees: f32) -> Vec3 {
    let elevation = (90.0 - angle_degrees).to_radians();
    Vec3::new(0.0, CAM_DISTANCE * elevation.sin(), CAM_DISTANCE * elevation.cos())
}

fn setup_camera(mut commands: Commands, camera_angle: Res<CameraAngle>) {
    commands.insert_resource(ClearColor(crate::palette::color("void")));
    let offset = camera_offset(camera_angle.degrees);
    commands.spawn((
        Name::new("MainCamera"),
        Camera3d::default(),
        Tonemapping::None,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1080.0,
            },
            far: 5000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(offset)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_arena(
    mut commands: Commands,
    balance: Res<GameBalance>,
    camera_angle: Res<CameraAngle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    let arena = &balance.arena;
    arena_size.width = arena.start_width;
    arena_size.height = arena.start_height;
    let start_hw = arena.start_width / 2.0;
    let start_hh = arena.start_height / 2.0;

    let wall_height = 200.0;
    let wall_thickness = 20.0;
    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    let walls = [
        ("NorthWall", WallSide::North, Vec3::new(0.0, wall_height / 2.0, -start_hh), Vec3::new(start_hw * 2.0 + wall_thickness, wall_height, wall_thickness)),
        ("SouthWall", WallSide::South, Vec3::new(0.0, wall_height / 2.0, start_hh), Vec3::new(start_hw * 2.0 + wall_thickness, wall_height, wall_thickness)),
        ("WestWall", WallSide::West, Vec3::new(-start_hw, wall_height / 2.0, 0.0), Vec3::new(wall_thickness, wall_height, start_hh * 2.0 + wall_thickness)),
        ("EastWall", WallSide::East, Vec3::new(start_hw, wall_height / 2.0, 0.0), Vec3::new(wall_thickness, wall_height, start_hh * 2.0 + wall_thickness)),
    ];

    for (name, side, pos, size) in walls {
        commands.spawn((
            Name::new(name),
            Wall,
            side,
            Transform::from_translation(pos),
            Collider::cuboid(size.x, size.y, size.z),
            CollisionMargin(5.0),
            RigidBody::Static,
            wall_layers,
        ));
    }

    let floor_radius = 40.0;
    let elevation = (90.0 - camera_angle.degrees).to_radians();
    let z_stretch = 1.0 / elevation.sin();

    commands.spawn((
        Name::new("ArenaFloor"),
        FloorParams { radius: floor_radius },
        Mesh3d(meshes.add(rounded_floor_mesh(arena.start_width, arena.start_height, floor_radius, z_stretch))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: crate::palette::color("background"),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, -0.01, 0.0)),
    ));
}

fn camera_follow(
    player_query: Query<&Transform, With<crate::player::Player>>,
    mut camera_query: Query<
        &mut Transform,
        (With<Camera3d>, Without<crate::player::Player>),
    >,
    camera_angle: Res<CameraAngle>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let player_2d = crate::coord::to_2d(player_transform.translation);
    let cx = player_2d.x;
    let cz = -player_2d.y;

    let look_at = Vec3::new(cx, 0.0, cz);
    let offset = camera_offset(camera_angle.degrees);
    camera_transform.translation = look_at + offset;
    let elevation = (90.0 - camera_angle.degrees).to_radians();
    let up = if elevation.sin() > 0.99 { Vec3::NEG_Z } else { Vec3::Y };
    *camera_transform = camera_transform.looking_at(look_at, up);
}

fn reset_arena_size(
    mut arena_size: ResMut<CurrentArenaSize>,
    balance: Res<GameBalance>,
) {
    arena_size.width = balance.arena.start_width;
    arena_size.height = balance.arena.start_height;
}

fn update_arena_size(
    run_state: Res<RunState>,
    balance: Res<GameBalance>,
    mut arena_size: ResMut<CurrentArenaSize>,
) {
    let arena = &balance.arena;
    let t = (run_state.elapsed / balance.wave.ramp_duration_secs).clamp(0.0, 1.0);
    let w = arena.start_width + t * (arena.width - arena.start_width);
    let h = arena.start_height + t * (arena.height - arena.start_height);
    if (arena_size.width - w).abs() > 0.1 || (arena_size.height - h).abs() > 0.1 {
        arena_size.width = w;
        arena_size.height = h;
    }
}

fn update_walls(
    arena_size: Option<Res<CurrentArenaSize>>,
    mut query: Query<(&WallSide, &mut Transform, &mut Collider)>,
) {
    let Some(arena_size) = arena_size else { return };
    if !arena_size.is_changed() {
        return;
    }
    let half_w = arena_size.half_w();
    let half_h = arena_size.half_h();
    let wall_thickness = 20.0;
    let wall_height = 200.0;

    for (side, mut transform, mut collider) in &mut query {
        match side {
            WallSide::North => {
                transform.translation.z = -half_h;
                *collider = Collider::cuboid(half_w * 2.0 + wall_thickness, wall_height, wall_thickness);
            }
            WallSide::South => {
                transform.translation.z = half_h;
                *collider = Collider::cuboid(half_w * 2.0 + wall_thickness, wall_height, wall_thickness);
            }
            WallSide::West => {
                transform.translation.x = -half_w;
                *collider = Collider::cuboid(wall_thickness, wall_height, half_h * 2.0 + wall_thickness);
            }
            WallSide::East => {
                transform.translation.x = half_w;
                *collider = Collider::cuboid(wall_thickness, wall_height, half_h * 2.0 + wall_thickness);
            }
        }
    }
}

fn update_target_count(
    run_state: Res<RunState>,
    mut wave_state: ResMut<WaveState>,
    balance: Res<GameBalance>,
) {
    let wb = &balance.wave;
    let t = (run_state.elapsed / wb.ramp_duration_secs).clamp(0.0, 1.0);
    let target = wb.start_enemies as f32 + t * (wb.max_enemies - wb.start_enemies) as f32;
    wave_state.max_concurrent = target.round() as u32;
}

fn spawn_enemies(
    mut commands: Commands,
    mut wave_state: ResMut<WaveState>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    mobs_balance: Res<MobsBalance>,
    balance: Res<GameBalance>,
    run_state: Res<RunState>,
    circle_mesh: Res<SummoningCircleMesh>,
    circle_material: Res<SummoningCircleMaterial>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spawn_pool: Res<EnemySpawnPool>,
    arena_size: Res<CurrentArenaSize>,
) {
    let active_enemies = wave_state.spawned_count - wave_state.killed_count;
    let deficit = wave_state.max_concurrent.saturating_sub(active_enemies);
    if deficit == 0 {
        return;
    }

    let player_pos = player_query
        .single()
        .map(|t| crate::coord::to_2d(t.translation))
        .unwrap_or(Vec2::ZERO);

    let safe_radius_sq = balance.wave.safe_spawn_radius * balance.wave.safe_spawn_radius;
    let margin = 30.0;
    let hw = arena_size.half_w() - margin;
    let hh = arena_size.half_h() - margin;
    let mut rng = rand::rng();

    let mut extra_modifiers: Vec<(Stat, f32)> = Vec::new();
    let elapsed = run_state.elapsed;
    if elapsed > 0.0 {
        let hp_bonus = elapsed * balance.run.hp_scale_per_sec;
        let dmg_bonus = elapsed * balance.run.dmg_scale_per_sec;
        extra_modifiers.push((Stat::MaxLifeIncreased, hp_bonus));
        extra_modifiers.push((Stat::PhysicalDamageIncreased, dmg_bonus));
    }

    for _ in 0..deficit {
        let (x, y) = {
            let mut attempts = 0;
            loop {
                let x = rng.random_range(-hw..hw);
                let y = rng.random_range(-hh..hh);
                let pos = Vec2::new(x, y);
                attempts += 1;
                if attempts > 100
                    || (is_inside_arena(pos, margin, &arena_size)
                        && pos.distance_squared(player_pos) > safe_radius_sq)
                {
                    break (x, y);
                }
            }
        };

        let active = spawn_pool.active_kinds();
        if active.is_empty() {
            break;
        }
        let kind = active[rng.random_range(0..active.len())];
        let circle_size = kind.size(&mobs_balance);
        let ground = crate::coord::ground_pos(Vec2::new(x, y));

        let is_ghost = matches!(kind, MobKind::Ghost);
        let mat_handle = if is_ghost {
            let cloned = materials.get(&circle_material.0).cloned();
            if let Some(base_mat) = cloned {
                MeshMaterial3d(materials.add(base_mat))
            } else {
                MeshMaterial3d(circle_material.0.clone())
            }
        } else {
            MeshMaterial3d(circle_material.0.clone())
        };

        let mut entity_commands = commands.spawn((
            Name::new("SummoningCircle"),
            Mesh3d(circle_mesh.0.clone()),
            mat_handle,
            Transform::from_translation(ground + Vec3::Y * 0.02)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::ZERO),
            SummoningCircle::new(kind, circle_size, extra_modifiers.clone()),
            DespawnOnExit(WavePhase::Combat),
        ));

        if is_ghost {
            use crate::actors::components::mob::ghost_transparency::{GhostTransparency, GhostAlpha};
            entity_commands.insert((
                GhostTransparency {
                    visible_distance: mobs_balance.ghost.visible_distance,
                    invisible_distance: mobs_balance.ghost.invisible_distance,
                },
                GhostAlpha { value: 0.0 },
            ));
        }
        wave_state.spawned_count += 1;
        wave_state.summoning_count += 1;
    }
}

fn is_inside_arena(pos: Vec2, margin: f32, arena: &CurrentArenaSize) -> bool {
    let hw = arena.half_w() - margin;
    let hh = arena.half_h() - margin;
    pos.x.abs() <= hw && pos.y.abs() <= hh
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

fn update_floor_mesh(
    camera_angle: Res<CameraAngle>,
    arena_size: Option<Res<CurrentArenaSize>>,
    query: Query<(&FloorParams, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(arena_size) = arena_size else { return };
    if !camera_angle.is_changed() && !arena_size.is_changed() {
        return;
    }
    let elevation = (90.0 - camera_angle.degrees).to_radians();
    let z_stretch = 1.0 / elevation.sin();
    for (params, mesh_handle) in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = rounded_floor_mesh(arena_size.width, arena_size.height, params.radius, z_stretch);
        }
    }
}

fn rounded_floor_mesh(width: f32, height: f32, radius: f32, z_stretch: f32) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};

    let hw = width / 2.0;
    let hh = height / 2.0;
    let r = radius.min(hw).min(hh);
    let rz = r * z_stretch;
    let segments = 8u32;

    let corner_centers = [
        [hw - r, -(hh - rz)],
        [hw - r, hh - rz],
        [-(hw - r), hh - rz],
        [-(hw - r), -(hh - rz)],
    ];

    let mut outline = Vec::new();
    for (i, center) in corner_centers.iter().enumerate() {
        let start = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::FRAC_PI_2;
        for j in 0..segments {
            let angle = start + std::f32::consts::FRAC_PI_2 * j as f32 / segments as f32;
            outline.push([center[0] + r * angle.cos(), center[1] + rz * angle.sin()]);
        }
    }

    let n = outline.len() as u32;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(1 + outline.len());
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(1 + outline.len());
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(1 + outline.len());
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for p in &outline {
        positions.push([p[0], 0.0, p[1]]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + p[0] / width, 0.5 + p[1] / height]);
    }

    for i in 0..n {
        indices.extend_from_slice(&[0, 1 + (i + 1) % n, 1 + i]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
