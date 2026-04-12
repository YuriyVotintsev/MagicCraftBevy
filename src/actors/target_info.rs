use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetInfo {
    pub entity: Option<Entity>,
    pub position: Option<Vec2>,
    pub direction: Option<Vec2>,
}

impl TargetInfo {
    pub const EMPTY: Self = Self { entity: None, position: None, direction: None };

    pub fn from_entity_and_position(entity: Entity, position: Vec2) -> Self {
        Self { entity: Some(entity), position: Some(position), direction: None }
    }

    #[allow(dead_code)]
    pub fn from_position(position: Vec2) -> Self {
        Self { entity: None, position: Some(position), direction: None }
    }

    pub fn from_direction(direction: Vec2) -> Self {
        Self { entity: None, position: None, direction: Some(direction) }
    }
}
