use bevy::prelude::{Sprite as BevySprite, *};
use magic_craft_macros::blueprint_component;
use serde::Deserialize;

use crate::palette;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(from = "SpriteColorInput")]
pub struct SpriteColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub flash: Option<(f32, f32, f32)>,
}

impl Default for SpriteColor {
    fn default() -> Self {
        Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0, flash: None }
    }
}

impl From<(f32, f32, f32, f32)> for SpriteColor {
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Self { r: t.0, g: t.1, b: t.2, a: t.3, flash: None }
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
                let flash = palette::flash_lookup(&name);
                Self { r, g, b, a: 1.0, flash }
            }
            SpriteColorInput::NamedWithAlpha(name, alpha) => {
                let (r, g, b) = palette::lookup(&name)
                    .unwrap_or_else(|| panic!("Unknown palette color: {name}"));
                let flash = palette::flash_lookup(&name);
                Self { r, g, b, a: alpha, flash }
            }
            SpriteColorInput::Rgba(r, g, b, a) => Self { r, g, b, a, flash: None },
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum SpriteShape {
    #[default]
    Square,
    Circle,
    Triangle,
    Capsule,
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
    #[raw(default = 0.5)]
    pub elevation: ScalarExpr,
    #[raw(default = 0.5)]
    pub half_length: ScalarExpr,
}

#[derive(Component)]
pub struct CircleSprite {
    pub color: Color,
    pub flash_color: Option<Color>,
}

#[derive(Component)]
pub struct TriangleSprite {
    pub color: Color,
    pub flash_color: Option<Color>,
}

#[derive(Component)]
pub struct SquareSprite {
    pub color: Color,
    pub flash_color: Option<Color>,
}

#[derive(Component)]
pub struct CapsuleSprite {
    pub color: Color,
    pub flash_color: Option<Color>,
    pub half_length: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, (setup_sphere_mesh, setup_cube_mesh));
    app.add_systems(PostUpdate, (init_sprite, spawn_circle_visuals, spawn_triangle_visuals, spawn_square_visuals, spawn_capsule_visuals).chain());
}

fn init_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Sprite, Has<Transform>), Added<Sprite>>,
) {
    for (entity, sprite, has_transform) in &query {
        let color = Color::srgba(sprite.color.r, sprite.color.g, sprite.color.b, sprite.color.a);
        let flash_color = sprite.color.flash.map(|(r, g, b)| Color::srgb(r, g, b));

        if !has_transform {
            let pos = crate::coord::ground_pos(sprite.position);
            commands.entity(entity).insert(
                Transform::from_translation(Vec3::new(pos.x, sprite.elevation, pos.z)),
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
                    commands.entity(entity).insert(SquareSprite { color, flash_color });
                }
                SpriteShape::Circle => {
                    commands.entity(entity).insert(CircleSprite { color, flash_color });
                }
                SpriteShape::Triangle => {
                    commands.entity(entity).insert(TriangleSprite { color, flash_color });
                }
                SpriteShape::Capsule => {
                    commands.entity(entity).insert(CapsuleSprite {
                        color,
                        flash_color,
                        half_length: sprite.half_length,
                    });
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

fn spawn_capsule_visuals(
    mut commands: Commands,
    query: Query<(Entity, &CapsuleSprite), Without<Mesh3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, capsule) in &query {
        let mesh = meshes.add(Capsule3d::new(0.5, capsule.half_length));
        let material = materials.add(StandardMaterial {
            base_color: capsule.color,
            unlit: true,
            alpha_mode: if capsule.color.alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        });
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
        ));
    }
}
