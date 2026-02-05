use avian2d::prelude::{Collider as AvianCollider, *};
use bevy::prelude::*;
use magic_craft_macros::ability_component;
use serde::Deserialize;

use crate::physics::GameLayer;
use crate::Faction;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Shape {
    Circle,
}

#[ability_component]
pub struct Collider {
    pub shape: Shape,
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
