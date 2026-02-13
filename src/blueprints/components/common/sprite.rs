use bevy::prelude::{Sprite as BevySprite, *};
use magic_craft_macros::blueprint_component;
use serde::Deserialize;

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
}

#[derive(Component)]
pub struct CircleSprite {
    pub color: Color,
}

#[derive(Component)]
pub struct TriangleSprite {
    pub color: Color,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, (setup_circle_mesh, setup_triangle_mesh));
    app.add_systems(PostUpdate, (init_sprite, spawn_circle_visuals, spawn_triangle_visuals).chain());
}

fn init_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Sprite, Has<Transform>), Added<Sprite>>,
) {
    for (entity, sprite, has_transform) in &query {
        let color = Color::srgba(sprite.color.0, sprite.color.1, sprite.color.2, sprite.color.3);
        let size = Vec2::splat(sprite.scale);

        if !has_transform {
            commands.entity(entity).insert(
                Transform::from_translation(sprite.position.extend(0.0)),
            );
        }

        if let Some(ref image_path) = sprite.image {
            commands.entity(entity).insert(BevySprite {
                image: asset_server.load(image_path.clone()),
                color,
                custom_size: Some(size),
                ..default()
            });
        } else {
            match sprite.shape {
                SpriteShape::Square => {
                    commands.entity(entity).insert(BevySprite {
                        color,
                        custom_size: Some(size),
                        ..default()
                    });
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
struct CircleMeshHandle(Handle<Mesh>);

fn setup_circle_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Circle::new(0.5));
    commands.insert_resource(CircleMeshHandle(mesh));
}

fn spawn_circle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &CircleSprite), Without<Mesh2d>>,
    circle_mesh: Option<Res<CircleMeshHandle>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(circle_mesh) = circle_mesh else { return };

    for (entity, circle) in &query {
        let material = materials.add(ColorMaterial::from_color(circle.color));
        commands.entity(entity).insert((
            Mesh2d(circle_mesh.0.clone()),
            MeshMaterial2d(material),
        ));
    }
}

#[derive(Resource)]
struct TriangleMeshHandle(Handle<Mesh>);

fn setup_triangle_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(RegularPolygon::new(0.5, 3));
    commands.insert_resource(TriangleMeshHandle(mesh));
}

fn spawn_triangle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &TriangleSprite), Without<Mesh2d>>,
    triangle_mesh: Option<Res<TriangleMeshHandle>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(triangle_mesh) = triangle_mesh else { return };

    for (entity, triangle) in &query {
        let material = materials.add(ColorMaterial::from_color(triangle.color));
        commands.entity(entity).insert((
            Mesh2d(triangle_mesh.0.clone()),
            MeshMaterial2d(material),
        ));
    }
}
