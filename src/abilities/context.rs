use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct TargetInfo {
    pub entity: Option<Entity>,
    pub position: Option<Vec2>,
    pub direction: Option<Vec2>,
}

impl TargetInfo {
    pub const EMPTY: Self = Self {
        entity: None,
        position: None,
        direction: None,
    };

    pub fn from_entity_and_position(entity: Entity, position: Vec2) -> Self {
        Self {
            entity: Some(entity),
            position: Some(position),
            direction: None,
        }
    }

    pub fn from_position(position: Vec2) -> Self {
        Self {
            entity: None,
            position: Some(position),
            direction: None,
        }
    }

    pub fn from_direction(direction: Vec2) -> Self {
        Self {
            entity: None,
            position: None,
            direction: Some(direction),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvidedFields(u8);

impl ProvidedFields {
    pub const NONE: Self = Self(0);
    pub const SOURCE_ENTITY: Self = Self(0b0000_0001);
    pub const SOURCE_POSITION: Self = Self(0b0000_0010);
    pub const SOURCE_DIRECTION: Self = Self(0b0000_0100);
    pub const TARGET_ENTITY: Self = Self(0b0000_1000);
    pub const TARGET_POSITION: Self = Self(0b0001_0000);
    pub const TARGET_DIRECTION: Self = Self(0b0010_0000);
    pub const ALL: Self = Self(0b0011_1111);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn missing(self, required: Self) -> Self {
        Self(required.0 & !self.0)
    }

    pub fn field_names(self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.0 & Self::SOURCE_ENTITY.0 != 0 {
            names.push("source.entity");
        }
        if self.0 & Self::SOURCE_POSITION.0 != 0 {
            names.push("source.position");
        }
        if self.0 & Self::SOURCE_DIRECTION.0 != 0 {
            names.push("source.direction");
        }
        if self.0 & Self::TARGET_ENTITY.0 != 0 {
            names.push("target.entity");
        }
        if self.0 & Self::TARGET_POSITION.0 != 0 {
            names.push("target.position");
        }
        if self.0 & Self::TARGET_DIRECTION.0 != 0 {
            names.push("target.direction");
        }
        names
    }
}
