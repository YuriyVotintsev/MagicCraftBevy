use bevy::prelude::*;

use crate::faction::Faction;
use super::context::TargetInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AbilityId(pub u32);

impl From<u32> for AbilityId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<AbilityId> for u32 {
    fn from(id: AbilityId) -> Self {
        id.0
    }
}

#[derive(Component)]
pub struct AbilityEntity;

#[derive(Component)]
pub struct AbilityInput {
    pub pressed: bool,
    pub target: TargetInfo,
}

#[derive(Component)]
pub struct AbilityCooldown {
    pub timer: f32,
}

#[derive(Component, Clone, Copy)]
pub struct AbilitySource {
    pub ability_id: AbilityId,
    pub caster: TargetInfo,
    pub caster_faction: Faction,
    pub source: TargetInfo,
    pub target: TargetInfo,
    pub index: usize,
    pub count: usize,
}
