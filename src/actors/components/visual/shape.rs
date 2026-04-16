use bevy::prelude::*;
use serde::Deserialize;

use crate::composite_scale::ScaleModifiers;
use crate::palette;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(from = "ShapeColorInput")]
pub struct ShapeColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub flash: Option<(f32, f32, f32)>,
}

impl Default for ShapeColor {
    fn default() -> Self {
        Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0, flash: None }
    }
}

impl From<(f32, f32, f32, f32)> for ShapeColor {
    fn from(t: (f32, f32, f32, f32)) -> Self {
        Self { r: t.0, g: t.1, b: t.2, a: t.3, flash: None }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ShapeColorInput {
    Named(String),
    NamedWithAlpha(String, f32),
    Rgba(f32, f32, f32, f32),
}

impl From<ShapeColorInput> for ShapeColor {
    fn from(input: ShapeColorInput) -> Self {
        match input {
            ShapeColorInput::Named(name) => {
                let (r, g, b) = palette::lookup(&name)
                    .unwrap_or_else(|| panic!("Unknown palette color: {name}"));
                let flash = palette::flash_lookup(&name);
                Self { r, g, b, a: 1.0, flash }
            }
            ShapeColorInput::NamedWithAlpha(name, alpha) => {
                let (r, g, b) = palette::lookup(&name)
                    .unwrap_or_else(|| panic!("Unknown palette color: {name}"));
                let flash = palette::flash_lookup(&name);
                Self { r, g, b, a: alpha, flash }
            }
            ShapeColorInput::Rgba(r, g, b, a) => Self { r, g, b, a, flash: None },
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum ShapeKind {
    #[default]
    Circle,
    Capsule,
    Disc,
}

#[derive(Component)]
pub struct Shape {
    pub color: ShapeColor,
    pub kind: ShapeKind,
    pub position: Vec2,
    pub elevation: f32,
    pub half_length: f32,
}

#[derive(Component)]
pub struct CircleShape {
    pub color: Color,
    pub flash_color: Option<Color>,
}


#[derive(Component)]
pub struct CapsuleShape {
    pub color: Color,
    pub flash_color: Option<Color>,
    pub half_length: f32,
}

#[derive(Component)]
pub struct DiscShape {
    pub color: Color,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, (setup_sphere_mesh, setup_disc_mesh));
    app.add_systems(PostUpdate, (init_shape, spawn_circle_visuals, spawn_capsule_visuals, spawn_disc_visuals).chain());
}

fn init_shape(
    mut commands: Commands,
    query: Query<(Entity, &Shape, Has<Transform>), Added<Shape>>,
) {
    for (entity, shape, has_transform) in &query {
        let color = Color::srgba(shape.color.r, shape.color.g, shape.color.b, shape.color.a);
        let flash_color = shape.color.flash.map(|(r, g, b)| Color::srgb(r, g, b));

        if !has_transform {
            let pos = crate::coord::ground_pos(shape.position);
            commands.entity(entity).insert(
                Transform::from_translation(Vec3::new(pos.x, shape.elevation, pos.z)),
            );
        }

        commands.entity(entity).insert(ScaleModifiers::default());

        match shape.kind {
            ShapeKind::Circle => {
                commands.entity(entity).insert(CircleShape { color, flash_color });
            }
            ShapeKind::Capsule => {
                commands.entity(entity).insert(CapsuleShape {
                    color,
                    flash_color,
                    half_length: shape.half_length,
                });
            }
            ShapeKind::Disc => {
                commands.entity(entity).insert(DiscShape { color });
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

fn spawn_circle_visuals(
    mut commands: Commands,
    query: Query<(Entity, &CircleShape), Without<Mesh3d>>,
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

#[derive(Resource)]
struct DiscMeshHandle(Handle<Mesh>);

fn setup_disc_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Circle::new(0.5));
    commands.insert_resource(DiscMeshHandle(mesh));
}

fn spawn_disc_visuals(
    mut commands: Commands,
    mut query: Query<(Entity, &DiscShape, &mut Transform), Without<Mesh3d>>,
    disc_mesh: Option<Res<DiscMeshHandle>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(disc_mesh) = disc_mesh else { return };

    for (entity, disc, mut transform) in &mut query {
        let material = materials.add(StandardMaterial {
            base_color: disc.color,
            unlit: true,
            alpha_mode: if disc.color.alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        });
        transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        commands.entity(entity).insert((
            Mesh3d(disc_mesh.0.clone()),
            MeshMaterial3d(material),
        ));
    }
}

fn spawn_capsule_visuals(
    mut commands: Commands,
    query: Query<(Entity, &CapsuleShape), Without<Mesh3d>>,
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
