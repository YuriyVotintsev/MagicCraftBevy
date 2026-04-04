use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::Tonemapping;
use rand::Rng;

use crate::balance::{ArenaBalance, GameBalance};
use crate::blueprints::components::common::health::Health;
use crate::blueprints::BlueprintRegistry;
use crate::physics::{GameLayer, Wall};
use crate::run::RunState;
use crate::schedule::GameSet;
use crate::stats::{DirtyStats, Modifiers, StatRegistry};
use crate::summoning::{SummoningAnimation, SummoningCircleMaterial, SummoningCircleMesh, DEFAULT_CIRCLE_SIZE};
use crate::wave::{WaveEnemy, WavePhase, WaveState};
use crate::Faction;
use crate::GameState;

pub const WINDOW_WIDTH: f32 = 1920.0;
pub const WINDOW_HEIGHT: f32 = 1080.0;

#[derive(Component)]
struct FloorParams {
    width: f32,
    height: f32,
    radius: f32,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraAngle>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::MainMenu),
                spawn_arena.run_if(not(any_with_component::<Wall>)),
            )
            .add_systems(
                Update,
                (update_target_count, spawn_enemies, tag_wave_enemies)
                    .chain()
                    .in_set(GameSet::Spawning)
                    .run_if(in_state(WavePhase::Combat))
                    .run_if(not(resource_exists::<crate::run::PlayerDying>)),
            )
            .add_systems(
                Update,
                apply_enemy_scaling
                    .in_set(GameSet::BlueprintActivation)
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                PostUpdate,
                camera_follow.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, update_floor_mesh);
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
) {
    let arena = &balance.arena;

    let half_w = arena.half_w();
    let half_h = arena.half_h();
    let wall_height = 200.0;
    let wall_thickness = 20.0;
    let wall_layers = CollisionLayers::new(GameLayer::Wall, LayerMask::ALL);

    let walls = [
        ("NorthWall", Vec3::new(0.0, wall_height / 2.0, -half_h), Vec3::new(half_w * 2.0 + wall_thickness, wall_height, wall_thickness)),
        ("SouthWall", Vec3::new(0.0, wall_height / 2.0, half_h), Vec3::new(half_w * 2.0 + wall_thickness, wall_height, wall_thickness)),
        ("WestWall", Vec3::new(-half_w, wall_height / 2.0, 0.0), Vec3::new(wall_thickness, wall_height, half_h * 2.0 + wall_thickness)),
        ("EastWall", Vec3::new(half_w, wall_height / 2.0, 0.0), Vec3::new(wall_thickness, wall_height, half_h * 2.0 + wall_thickness)),
    ];

    for (name, pos, size) in walls {
        commands.spawn((
            Name::new(name),
            Wall,
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
        FloorParams { width: arena.width, height: arena.height, radius: floor_radius },
        Mesh3d(meshes.add(rounded_floor_mesh(arena.width, arena.height, floor_radius, z_stretch))),
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
    enemies_query: Query<&Faction, With<Health>>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    blueprint_registry: Res<BlueprintRegistry>,
    balance: Res<GameBalance>,
    circle_mesh: Res<SummoningCircleMesh>,
    circle_material: Res<SummoningCircleMaterial>,
) {
    let alive_enemies = enemies_query.iter().filter(|f| **f == Faction::Enemy).count() as u32;
    let deficit = wave_state.max_concurrent.saturating_sub(alive_enemies + wave_state.summoning_count);
    if deficit == 0 {
        return;
    }

    let player_pos = player_query
        .single()
        .map(|t| crate::coord::to_2d(t.translation))
        .unwrap_or(Vec2::ZERO);

    let arena = &balance.arena;
    let safe_radius_sq = balance.wave.safe_spawn_radius * balance.wave.safe_spawn_radius;
    let margin = 30.0;
    let hw = arena.half_w() - margin;
    let hh = arena.half_h() - margin;
    let mut rng = rand::rng();

    for _ in 0..deficit {
        let (x, y) = {
            let mut attempts = 0;
            loop {
                let x = rng.random_range(-hw..hw);
                let y = rng.random_range(-hh..hh);
                let pos = Vec2::new(x, y);
                attempts += 1;
                if attempts > 100
                    || (is_inside_arena(pos, margin, arena)
                        && pos.distance_squared(player_pos) > safe_radius_sq)
                {
                    break (x, y);
                }
            }
        };

        let blueprint_name = match rng.random_range(0..3u32) {
            0 => "tower",
            1 => "jumper",
            _ => "slime_small",
        };

        if let Some(blueprint_id) = blueprint_registry.get_id(blueprint_name) {
            let ground = crate::coord::ground_pos(Vec2::new(x, y));

            let circle_entity = commands
                .spawn((
                    Name::new("SummoningCircle"),
                    Mesh3d(circle_mesh.0.clone()),
                    MeshMaterial3d(circle_material.0.clone()),
                    Transform::from_translation(ground + Vec3::Y * 0.02)
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_scale(Vec3::ZERO),
                    DespawnOnExit(WavePhase::Combat),
                ))
                .id();

            commands.spawn((
                Name::new("Enemy"),
                Transform::from_translation(ground),
                WaveEnemy,
                DespawnOnExit(WavePhase::Combat),
                SummoningAnimation::new(circle_entity, DEFAULT_CIRCLE_SIZE, blueprint_id),
            ));
            wave_state.spawned_count += 1;
            wave_state.summoning_count += 1;
        }
    }
}

fn apply_enemy_scaling(
    mut query: Query<(&mut Modifiers, &mut DirtyStats), Added<WaveEnemy>>,
    run_state: Res<RunState>,
    stat_registry: Res<StatRegistry>,
    balance: Res<GameBalance>,
) {
    let elapsed = run_state.elapsed;
    if elapsed <= 0.0 {
        return;
    }
    let hp_bonus = elapsed * balance.run.hp_scale_per_sec;
    let dmg_bonus = elapsed * balance.run.dmg_scale_per_sec;
    let hp_stat = stat_registry.get("max_life_increased");
    let dmg_stat = stat_registry.get("physical_damage_increased");

    for (mut modifiers, mut dirty) in &mut query {
        if let Some(stat) = hp_stat {
            modifiers.add(stat, hp_bonus);
            dirty.mark(stat);
        }
        if let Some(stat) = dmg_stat {
            modifiers.add(stat, dmg_bonus);
            dirty.mark(stat);
        }
    }
}

fn is_inside_arena(pos: Vec2, margin: f32, arena: &ArenaBalance) -> bool {
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
    query: Query<(&FloorParams, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !camera_angle.is_changed() {
        return;
    }
    let elevation = (90.0 - camera_angle.degrees).to_radians();
    let z_stretch = 1.0 / elevation.sin();
    for (params, mesh_handle) in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = rounded_floor_mesh(params.width, params.height, params.radius, z_stretch);
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
