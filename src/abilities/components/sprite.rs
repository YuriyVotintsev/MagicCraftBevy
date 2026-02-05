use bevy::prelude::{Sprite as BevySprite, *};
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{VecExpr, VecExprRaw};
use crate::abilities::spawn::SpawnContext;

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
    #[serde(default)]
    pub position: Option<VecExprRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub color: (f32, f32, f32, f32),
    pub shape: SpriteShape,
    pub position: Option<VecExpr>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            color: self.color,
            shape: self.shape,
            position: self.position.as_ref().map(|p| p.resolve(stat_registry)),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let fields = raw.position.as_ref().map(|p| p.required_fields()).unwrap_or(ProvidedFields::NONE);
    (fields, None)
}

#[derive(Component)]
pub struct Sprite {
    pub color: Color,
    pub shape: SpriteShape,
    pub position: Option<Vec2>,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let color = Color::srgba(def.color.0, def.color.1, def.color.2, def.color.3);

    let eval_ctx = ctx.eval_context();
    let position = match &def.position {
        Some(pos_expr) => Some(pos_expr.eval(&eval_ctx)),
        None => ctx.source.position,
    };

    commands.insert(Sprite { color, shape: def.shape, position });
}

#[derive(Component)]
pub struct CircleSprite {
    pub color: Color,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, setup_circle_mesh);
    app.add_systems(PostUpdate, (init_sprite, spawn_circle_visuals).chain());
}

fn init_sprite(
    mut commands: Commands,
    query: Query<(Entity, &Sprite), Added<Sprite>>,
) {
    for (entity, sprite) in &query {
        let transform = Transform::from_translation(
            sprite.position.unwrap_or(Vec2::ZERO).extend(0.0)
        );
        match sprite.shape {
            SpriteShape::Square => {
                commands.entity(entity).insert((
                    BevySprite {
                        color: sprite.color,
                        custom_size: Some(Vec2::ONE),
                        ..default()
                    },
                    transform,
                ));
            }
            SpriteShape::Circle => {
                commands.entity(entity).insert((CircleSprite { color: sprite.color }, transform));
            }
        }
    }
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
