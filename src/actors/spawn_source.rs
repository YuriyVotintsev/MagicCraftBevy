use bevy::prelude::*;

use crate::actors::target_info::TargetInfo;

#[derive(Component, Clone, Copy)]
pub struct SpawnSource {
    pub caster: TargetInfo,
    pub target: TargetInfo,
}

impl SpawnSource {
    pub fn from_caster(entity: Entity, pos: Vec2) -> Self {
        Self {
            caster: TargetInfo::from_entity_and_position(entity, pos),
            target: TargetInfo::EMPTY,
        }
    }

    pub fn with_target(entity: Entity, pos: Vec2, target: TargetInfo) -> Self {
        Self {
            caster: TargetInfo::from_entity_and_position(entity, pos),
            target,
        }
    }
}
