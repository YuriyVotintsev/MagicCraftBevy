use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::Tonemapping;

pub mod size;
pub use size::CurrentArenaSize;

use crate::balance::GameBalance;
use crate::faction::GameLayer;
use crate::GameState;

pub const WINDOW_WIDTH: f32 = 1920.0;
pub const WINDOW_HEIGHT: f32 = 1080.0;

#[derive(Component)]
struct FloorParams {
    radius: f32,
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
enum WallSide { North, South, West, East }

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraAngle>()
            .init_resource::<CurrentArenaSize>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                OnEnter(GameState::MainMenu),
                spawn_arena.run_if(not(any_with_component::<Wall>)),
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
    player_query: Query<&Transform, With<crate::actors::player::Player>>,
    mut camera_query: Query<
        &mut Transform,
        (With<Camera3d>, Without<crate::actors::player::Player>),
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
