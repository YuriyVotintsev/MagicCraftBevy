use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use serde::Deserialize;

use crate::arena::{CameraYaw, RenderSettings};
use crate::coords::vec2_to_3d;
use super::jump_walk_animation::JumpWalkAnimation;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(from = "(f32, f32, f32, f32)")]
pub struct SpriteColor(pub f32, pub f32, pub f32, pub f32);

impl Default for SpriteColor {
    fn default() -> Self {
        Self(1.0, 1.0, 1.0, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for SpriteColor {
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Self(t.0, t.1, t.2, t.3)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum SpriteShape {
    #[default]
    Square,
    Circle,
    Triangle,
}

#[blueprint_component]
pub struct Sprite {
    #[raw(default = SpriteColor::default())]
    pub color: SpriteColor,
    #[raw(default = SpriteShape::Square)]
    pub shape: SpriteShape,
    #[default_expr("source.position")]
    pub position: VecExpr,
    #[raw(default = 1.0)]
    pub scale: ScalarExpr,
    #[raw(default = None)]
    pub image: Option<String>,
    #[raw(default = false)]
    pub standing: bool,
}

#[derive(Component)]
pub struct FaceCamera;

#[derive(Component)]
pub struct StandingSprite;

#[derive(Component)]
pub struct CircleSprite {
    pub color: Color,
    pub standing: bool,
}

#[derive(Component)]
pub struct TriangleSprite {
    pub color: Color,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, (setup_circle_mesh, setup_standing_circle_mesh, setup_triangle_mesh));
    app.add_systems(PostUpdate, (init_sprite, spawn_circle_visuals, spawn_triangle_visuals).chain());
    app.add_systems(PostUpdate, (apply_sprite_tilt, face_camera_system));
}

fn init_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Sprite, Has<Transform>), Added<Sprite>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, sprite, has_transform) in &query {
        let color = Color::srgba(sprite.color.0, sprite.color.1, sprite.color.2, sprite.color.3);
        let size = sprite.scale;

        if !has_transform {
            commands.entity(entity).insert(
                Transform::from_translation(vec2_to_3d(sprite.position)),
            );
        }

        if let Some(ref image_path) = sprite.image {
            let mesh = meshes.add(
                Mesh::from(Rectangle::new(size, size))
                    .translated_by(Vec3::new(0.0, size / 2.0, 0.0)),
            );
            let material = materials.add(StandardMaterial {
                base_color: color,
                base_color_texture: Some(asset_server.load(image_path.clone())),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });
            commands.entity(entity).insert((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                StandingSprite,
            ));
        } else {
            match sprite.shape {
                SpriteShape::Square => {
                    let mesh = meshes.add(
                        Mesh::from(Rectangle::new(size, size))
                            .translated_by(Vec3::new(0.0, size / 2.0, 0.0)),
                    );
                    let material = materials.add(StandardMaterial {
                        base_color: color,
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    });
                    commands.entity(entity).insert((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        StandingSprite,
                    ));
                }
                SpriteShape::Circle => {
                    commands.entity(entity).insert(CircleSprite { color, standing: sprite.standing });
                }
                SpriteShape::Triangle => {
                    commands.entity(entity).insert(TriangleSprite { color });
                }
            }
        }
    }
}

#[derive(Resource)]
struct CircleMeshHandle(Handle<Mesh>);

#[derive(Resource)]
struct StandingCircleMeshHandle(Handle<Mesh>);

fn setup_circle_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(
        Mesh::from(Circle::new(0.5))
            .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    );
    commands.insert_resource(CircleMeshHandle(mesh));
}

fn setup_standing_circle_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Circle::new(0.5));
    commands.insert_resource(StandingCircleMeshHandle(mesh));
}

fn spawn_circle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &CircleSprite), Without<Mesh3d>>,
    circle_mesh: Option<Res<CircleMeshHandle>>,
    standing_circle_mesh: Option<Res<StandingCircleMeshHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(circle_mesh) = circle_mesh else { return };
    let Some(standing_circle_mesh) = standing_circle_mesh else { return };

    for (entity, circle) in &query {
        let material = materials.add(StandardMaterial {
            base_color: circle.color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        if circle.standing {
            commands.entity(entity).insert((
                Mesh3d(standing_circle_mesh.0.clone()),
                MeshMaterial3d(material),
                StandingSprite,
            ));
        } else {
            commands.entity(entity).insert((
                Mesh3d(circle_mesh.0.clone()),
                MeshMaterial3d(material),
            ));
        }
    }
}

#[derive(Resource)]
struct TriangleMeshHandle(Handle<Mesh>);

fn setup_triangle_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(
        Mesh::from(RegularPolygon::new(0.5, 3))
            .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    );
    commands.insert_resource(TriangleMeshHandle(mesh));
}

fn spawn_triangle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &TriangleSprite), Without<Mesh3d>>,
    triangle_mesh: Option<Res<TriangleMeshHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(triangle_mesh) = triangle_mesh else { return };

    for (entity, triangle) in &query {
        let material = materials.add(StandardMaterial {
            base_color: triangle.color,
            unlit: true,
            ..default()
        });
        commands.entity(entity).insert((
            Mesh3d(triangle_mesh.0.clone()),
            MeshMaterial3d(material),
        ));
    }
}

fn apply_sprite_tilt(
    settings: Res<RenderSettings>,
    yaw: Res<CameraYaw>,
    mut query: Query<&mut Transform, (With<StandingSprite>, Without<JumpWalkAnimation>)>,
) {
    let tilt = Quat::from_rotation_x(-settings.sprite_tilt.to_radians());
    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_y(yaw.0) * tilt;
    }
}

fn face_camera_system(
    cam: Query<&GlobalTransform, With<Camera3d>>,
    mut query: Query<(&GlobalTransform, &mut Transform), (With<FaceCamera>, Without<Camera3d>)>,
) {
    let Ok(cam_gt) = cam.single() else { return };
    let cam_pos = cam_gt.translation();
    for (gt, mut transform) in &mut query {
        let world_pos = gt.translation();
        let dir = cam_pos - world_pos;
        if dir.x * dir.x + dir.z * dir.z > 0.0 {
            let angle = dir.x.atan2(dir.z);
            transform.rotation = Quat::from_rotation_y(angle);
        }
    }
}
