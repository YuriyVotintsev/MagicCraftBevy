use bevy::prelude::*;

use crate::faction::Faction;
use super::context::TargetInfo;
pub use super::ids::AbilityId;

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
