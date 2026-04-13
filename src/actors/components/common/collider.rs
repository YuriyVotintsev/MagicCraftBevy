use avian3d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use serde::Deserialize;

use crate::Faction;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    PlayerProjectile,
    EnemyProjectile,
    Wall,
}

impl Faction {
    pub fn enemy_layer(self) -> GameLayer {
        match self {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Shape {
    Circle,
    Rectangle,
}

#[derive(Component)]
pub struct Collider {
    pub shape: Shape,
    pub sensor: bool,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, init_collider);
}

fn init_collider(
    mut commands: Commands,
    query: Query<(Entity, &Collider, &Faction), Added<Collider>>,
) {
    for (entity, collider, faction) in &query {
        let avian_collider = match collider.shape {
            Shape::Circle => AvianCollider::cylinder(0.5, 0.2),
            Shape::Rectangle => AvianCollider::cuboid(1.0, 0.2, 1.0),
        };

        if collider.sensor {
            let layers = match faction {
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
        } else {
            let layers = match faction {
                Faction::Player => CollisionLayers::new(
                    GameLayer::Player,
                    [GameLayer::Enemy, GameLayer::EnemyProjectile, GameLayer::Wall, GameLayer::Player],
                ),
                Faction::Enemy => CollisionLayers::new(
                    GameLayer::Enemy,
                    [GameLayer::Player, GameLayer::PlayerProjectile, GameLayer::Wall, GameLayer::Enemy],
                ),
            };
            commands.entity(entity).insert((avian_collider, CollisionEventsEnabled, layers));
        }
    }
}
