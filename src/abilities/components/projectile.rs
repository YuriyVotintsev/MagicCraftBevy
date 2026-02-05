use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
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

pub fn required_fields_and_nested(_raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    (ProvidedFields::NONE, None)
}

#[derive(Component)]
pub struct Projectile;

pub fn spawn(commands: &mut EntityCommands, _def: &Def, _ctx: &SpawnContext) {
    commands.insert((Name::new("Projectile"), Projectile));
}

pub fn register_systems(_app: &mut App) {}
