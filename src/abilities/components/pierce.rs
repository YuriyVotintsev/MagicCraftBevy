use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DefRaw {
    #[serde(default)]
    pub count: Option<ParamValueRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub count: Option<ParamValue>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            count: self.count.as_ref().map(|p| resolve_param_value(p, stat_registry)),
        }
    }
}

#[derive(Component)]
pub enum Pierce {
    Count(u32),
    Infinite,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let pierce = match &def.count {
        Some(param) => Pierce::Count(param.evaluate_i32(ctx.stats) as u32),
        None => Pierce::Infinite,
    };
    commands.insert(pierce);
}

pub fn register_systems(_app: &mut App) {}
