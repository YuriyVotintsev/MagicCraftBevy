use bevy::prelude::*;

use crate::actors::target_info::TargetInfo;
use crate::faction::Faction;

#[derive(Component, Clone, Copy)]
pub struct SpawnSource {
    pub caster: TargetInfo,
    pub caster_faction: Faction,
    pub target: TargetInfo,
}
