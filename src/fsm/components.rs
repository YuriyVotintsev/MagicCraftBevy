use bevy::prelude::*;

use super::types::ColliderShape;

#[derive(Component)]
pub struct CurrentState(pub String);

#[derive(Component)]
pub struct MobType(pub String);

#[derive(Component, Clone)]
pub struct Collider {
    #[allow(dead_code)]
    pub shape: ColliderShape,
    pub size: f32,
}
