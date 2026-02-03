use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::spawn::SpawnContext;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DefRaw;

#[derive(Debug, Clone, Default)]
pub struct Def;

impl DefRaw {
    pub fn resolve(&self, _stat_registry: &crate::stats::StatRegistry) -> Def {
        Def
    }
}

#[derive(Component)]
pub struct Stationary;

pub fn spawn(commands: &mut EntityCommands, _def: &Def, _ctx: &SpawnContext) {
    commands.insert(Stationary);
}

pub fn register_systems(_app: &mut App) {}
