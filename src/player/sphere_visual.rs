use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use bevy::asset::RenderAssetUsages;
use bevy::camera::ScalingMode;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::blueprints::components::common::sprite::CircleSprite;
use crate::palette;
use super::Player;

#[derive(Component)]
struct PlayerSphere;

#[derive(Component)]
struct SphereCamera;

#[derive(Resource)]
struct SphereMeshHandle(Handle<Mesh>);

#[derive(Resource)]
struct SphereMaterialHandle(Handle<StandardMaterial>);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, setup_sphere_resources);
    app.add_systems(PostUpdate, (
        parent_sphere_camera,
        attach_sphere_to_player,
        rotate_sphere,
    ));
}

fn setup_sphere_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let texture_handle = create_eye_texture(&mut images);

    let mesh_handle = meshes.add(Sphere::new(0.5).mesh().uv(32, 18));

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        unlit: true,
        ..default()
    });

    commands.insert_resource(SphereMeshHandle(mesh_handle));
    commands.insert_resource(SphereMaterialHandle(material_handle));

    commands.spawn((
        Name::new("SphereCamera"),
        SphereCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Tonemapping::None,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1080.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, 100.0),
        RenderLayers::layer(1),
    ));
}

fn create_eye_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let size = 256u32;
    let (pr, pg, pb) = palette::lookup("player").unwrap_or((0.224, 0.416, 0.482));

    let bg_r = (pr * 255.0) as u8;
    let bg_g = (pg * 255.0) as u8;
    let bg_b = (pb * 255.0) as u8;

    let mut data = vec![0u8; (size * size * 4) as usize];

    for i in 0..(size * size) as usize {
        data[i * 4] = bg_r;
        data[i * 4 + 1] = bg_g;
        data[i * 4 + 2] = bg_b;
        data[i * 4 + 3] = 255;
    }

    let eye_radius_sq = 16.0f32 * 16.0;
    let eyes = [(108.0f32, 100.0f32), (148.0f32, 100.0f32)];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) as usize * 4;
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;

            for &(ex, ey) in &eyes {
                let dx = fx - ex;
                let dy = fy - ey;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= eye_radius_sq {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                    data[idx + 3] = 255;
                }
            }
        }
    }

    images.add(Image::new(
        Extent3d { width: size, height: size, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

fn attach_sphere_to_player(
    mut commands: Commands,
    player_query: Query<&Children, With<Player>>,
    circle_query: Query<Entity, (With<CircleSprite>, Without<PlayerSphere>)>,
    sphere_exists: Query<(), With<PlayerSphere>>,
    mesh: Option<Res<SphereMeshHandle>>,
    material: Option<Res<SphereMaterialHandle>>,
) {
    if !sphere_exists.is_empty() { return }
    let (Some(mesh), Some(material)) = (mesh, material) else { return };

    for player_children in &player_query {
        for child in player_children.iter() {
            if circle_query.get(child).is_err() { continue }

            commands.entity(child).insert(RenderLayers::layer(2));

            let initial_rotation =
                Quat::from_rotation_x(FRAC_PI_4)
                * Quat::from_rotation_y(-FRAC_PI_2)
                * Quat::from_rotation_x(-FRAC_PI_2);
            commands.entity(child).with_child((
                PlayerSphere,
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(material.0.clone()),
                Transform::from_rotation(initial_rotation),
                RenderLayers::layer(1),
            ));
            return;
        }
    }
}

fn parent_sphere_camera(
    mut commands: Commands,
    cam2d: Query<Entity, With<Camera2d>>,
    cam3d: Query<Entity, (With<SphereCamera>, Without<ChildOf>)>,
) {
    let (Ok(cam2d), Ok(cam3d)) = (cam2d.single(), cam3d.single()) else { return };
    commands.entity(cam2d).add_child(cam3d);
}

fn rotate_sphere(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<PlayerSphere>>,
) {
    for mut transform in &mut query {
        transform.rotate_local_z(0.5 * time.delta_secs());
    }
}
