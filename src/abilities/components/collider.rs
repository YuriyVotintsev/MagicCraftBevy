use avian2d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::spawn::SpawnContext;
use crate::physics::GameLayer;
use crate::Faction;

#[derive(Debug, Clone, Deserialize)]
pub enum ShapeRaw {
    Circle,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Circle,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub shape: ShapeRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub shape: Shape,
}

impl DefRaw {
    pub fn resolve(&self, _stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            shape: match &self.shape {
                ShapeRaw::Circle => Shape::Circle,
            },
        }
    }
}

pub fn required_fields_and_nested(_raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    (ProvidedFields::NONE, None)
}

#[derive(Component)]
pub struct Collider {
    pub shape: Shape,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, _ctx: &SpawnContext) {
    commands.insert(Collider { shape: def.shape.clone() });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_collider);
}

fn init_collider(
    mut commands: Commands,
    query: Query<(Entity, &Collider, &crate::abilities::AbilitySource), Added<Collider>>,
) {
    for (entity, collider, source) in &query {
        let avian_collider = match collider.shape {
            Shape::Circle => AvianCollider::circle(1.0),
        };

        let layers = match source.caster_faction {
            Faction::Player => CollisionLayers::new(
                GameLayer::PlayerProjectile,
                [GameLayer::Enemy, GameLayer::Wall],
            ),
            Faction::Enemy => CollisionLayers::new(
                GameLayer::EnemyProjectile,
                [GameLayer::Player, GameLayer::Wall],
            ),
        };

        commands.entity(entity).insert((avian_collider, Sensor, CollisionEventsEnabled, layers));
    }
}
