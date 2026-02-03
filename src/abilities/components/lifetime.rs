use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::Lifetime;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw(pub ParamValueRaw);

#[derive(Debug, Clone)]
pub struct Def(pub ParamValue);

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def(resolve_param_value(&self.0, stat_registry))
    }
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let remaining = def.0.evaluate_f32(ctx.stats);
    commands.insert(Lifetime { remaining });
}

pub fn register_systems(_app: &mut App) {}
