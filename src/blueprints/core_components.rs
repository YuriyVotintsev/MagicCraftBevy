use bevy::prelude::*;

use crate::faction::Faction;
use super::context::TargetInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlueprintId(pub u32);

impl From<u32> for BlueprintId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<BlueprintId> for u32 {
    fn from(id: BlueprintId) -> Self {
        id.0
    }
}

#[derive(Component)]
pub struct BlueprintEntity;

#[derive(Component)]
pub struct BlueprintActivationInput {
    pub pressed: bool,
    pub target: TargetInfo,
}

#[derive(Component)]
pub struct BlueprintActivationCooldown {
    pub timer: f32,
}

#[derive(Component)]
pub struct TrackedSpawns {
    pub entities: Vec<Entity>,
}

#[derive(Component, Clone, Copy)]
pub struct SpawnSource {
    pub blueprint_id: BlueprintId,
    pub caster: TargetInfo,
    pub caster_faction: Faction,
    pub source: TargetInfo,
    pub target: TargetInfo,
    pub index: usize,
    pub count: usize,
}
