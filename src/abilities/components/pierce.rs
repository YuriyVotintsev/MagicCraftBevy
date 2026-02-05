use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DefRaw {
    #[serde(default)]
    pub count: Option<ScalarExprRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub count: Option<ScalarExpr>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            count: self.count.as_ref().map(|p| p.resolve(stat_registry)),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let fields = raw.count.as_ref().map(|c| c.required_fields()).unwrap_or(ProvidedFields::NONE);
    (fields, None)
}

#[derive(Component)]
pub enum Pierce {
    Count(u32),
    Infinite,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let pierce = match &def.count {
        Some(param) => Pierce::Count(param.eval(&ctx.eval_context()) as u32),
        None => Pierce::Infinite,
    };
    commands.insert(pierce);
}

pub fn register_systems(_app: &mut App) {}
