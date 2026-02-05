use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw(pub ScalarExprRaw);

#[derive(Debug, Clone)]
pub struct Def(pub ScalarExpr);

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def(self.0.resolve(stat_registry))
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    (raw.0.required_fields(), None)
}

#[derive(Component)]
pub struct Speed(pub f32);

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let value = def.0.eval(&ctx.eval_context());
    commands.insert(Speed(value));
}

pub fn register_systems(_app: &mut App) {}
