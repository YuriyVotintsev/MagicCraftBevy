use bevy::prelude::{Sprite as BevySprite, *};
use magic_craft_macros::blueprint_component;
use serde::Deserialize;

use crate::palette;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(from = "SpriteColorInput")]
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

#[derive(Deserialize)]
#[serde(untagged)]
enum SpriteColorInput {
    Named(String),
    NamedWithAlpha(String, f32),
    Rgba(f32, f32, f32, f32),
}

impl From<SpriteColorInput> for SpriteColor {
    fn from(input: SpriteColorInput) -> Self {
        match input {
            SpriteColorInput::Named(name) => {
                let (r, g, b) = palette::lookup(&name)
                    .unwrap_or_else(|| panic!("Unknown palette color: {name}"));
                Self(r, g, b, 1.0)
            }
            SpriteColorInput::NamedWithAlpha(name, alpha) => {
                let (r, g, b) = palette::lookup(&name)
                    .unwrap_or_else(|| panic!("Unknown palette color: {name}"));
                Self(r, g, b, alpha)
            }
            SpriteColorInput::Rgba(r, g, b, a) => Self(r, g, b, a),
        }
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
}

#[derive(Component)]
pub struct CircleSprite {
    pub color: Color,
}

#[derive(Component)]
pub struct TriangleSprite {
    pub color: Color,
}

#[derive(Component)]
pub struct SquareSprite {
    pub color: Color,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, (setup_sphere_mesh, setup_cube_mesh));
    app.add_systems(PostUpdate, (init_sprite, spawn_circle_visuals, spawn_triangle_visuals, spawn_square_visuals).chain());
}

fn init_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Sprite, Has<Transform>), Added<Sprite>>,
) {
    for (entity, sprite, has_transform) in &query {
        let color = Color::srgba(sprite.color.0, sprite.color.1, sprite.color.2, sprite.color.3);

        if !has_transform {
            let pos = crate::coord::ground_pos(sprite.position);
            commands.entity(entity).insert(
                Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z)),
            );
        }

        if let Some(ref image_path) = sprite.image {
            commands.entity(entity).insert(BevySprite {
                image: asset_server.load(image_path.clone()),
                color,
                custom_size: Some(Vec2::splat(sprite.scale)),
                ..default()
            });
        } else {
            match sprite.shape {
                SpriteShape::Square => {
                    commands.entity(entity).insert(SquareSprite { color });
                }
                SpriteShape::Circle => {
                    commands.entity(entity).insert(CircleSprite { color });
                }
                SpriteShape::Triangle => {
                    commands.entity(entity).insert(TriangleSprite { color });
                }
            }
        }
    }
}

#[derive(Resource)]
struct SphereMeshHandle(Handle<Mesh>);

fn setup_sphere_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Sphere::new(0.5));
    commands.insert_resource(SphereMeshHandle(mesh));
}

#[derive(Resource)]
struct CubeMeshHandle(Handle<Mesh>);

fn setup_cube_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.insert_resource(CubeMeshHandle(mesh));
}

fn spawn_circle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &CircleSprite), Without<Mesh3d>>,
    sphere_mesh: Option<Res<SphereMeshHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(sphere_mesh) = sphere_mesh else { return };

    for (entity, circle) in &query {
        let material = materials.add(StandardMaterial {
            base_color: circle.color,
            unlit: true,
            alpha_mode: if circle.color.alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        });
        commands.entity(entity).insert((
            Mesh3d(sphere_mesh.0.clone()),
            MeshMaterial3d(material),
        ));
    }
}

fn spawn_triangle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &TriangleSprite), Without<Mesh3d>>,
    sphere_mesh: Option<Res<SphereMeshHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(sphere_mesh) = sphere_mesh else { return };

    for (entity, triangle) in &query {
        let material = materials.add(StandardMaterial {
            base_color: triangle.color,
            unlit: true,
            alpha_mode: if triangle.color.alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        });
        commands.entity(entity).insert((
            Mesh3d(sphere_mesh.0.clone()),
            MeshMaterial3d(material),
        ));
    }
}

fn spawn_square_visuals(
    mut commands: Commands,
    query: Query<(Entity, &SquareSprite), Without<Mesh3d>>,
    cube_mesh: Option<Res<CubeMeshHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(cube_mesh) = cube_mesh else { return };

    for (entity, square) in &query {
        let material = materials.add(StandardMaterial {
            base_color: square.color,
            unlit: true,
            alpha_mode: if square.color.alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        });
        commands.entity(entity).insert((
            Mesh3d(cube_mesh.0.clone()),
            MeshMaterial3d(material),
        ));
    }
}
