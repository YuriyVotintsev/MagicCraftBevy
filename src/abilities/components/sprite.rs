use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::spawn::SpawnContext;
use crate::abilities::Target;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub enum SpriteShape {
    #[default]
    Square,
    Circle,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub color: (f32, f32, f32, f32),
    #[serde(default)]
    pub shape: SpriteShape,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub color: (f32, f32, f32, f32),
    pub shape: SpriteShape,
}

impl DefRaw {
    pub fn resolve(&self, _stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            color: self.color,
            shape: self.shape,
        }
    }
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let color = Color::srgba(def.color.0, def.color.1, def.color.2, def.color.3);

    let position = match ctx.source {
        Target::Point(p) => p,
        _ => Vec3::ZERO,
    };

    match def.shape {
        SpriteShape::Square => {
            commands.insert((
                Sprite {
                    color,
                    custom_size: Some(Vec2::ONE),
                    ..default()
                },
                Transform::from_translation(position),
            ));
        }
        SpriteShape::Circle => {
            commands.insert((
                CircleSprite { color },
                Transform::from_translation(position),
            ));
        }
    }
}

#[derive(Component)]
pub struct CircleSprite {
    pub color: Color,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, setup_circle_mesh);
    app.add_systems(PostUpdate, spawn_circle_visuals);
}

#[derive(Resource)]
struct CircleMeshHandle(Handle<Mesh>);

fn setup_circle_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Circle::new(1.0));
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
