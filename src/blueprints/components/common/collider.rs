use avian2d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use serde::Deserialize;

use crate::physics::GameLayer;
use crate::Faction;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Shape {
    Circle,
    Rectangle,
}

#[blueprint_component]
pub struct Collider {
    pub shape: Shape,
    #[raw(default = true)]
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
            Shape::Circle => AvianCollider::circle(1.0),
            Shape::Rectangle => AvianCollider::rectangle(2.0, 2.0),
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
