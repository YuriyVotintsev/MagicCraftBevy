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
pub struct Size(pub f32);

pub fn insert_component(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let size = def.0.eval(&ctx.eval_context());
    commands.insert(Size(size));
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, sync_size_to_scale);
}

fn sync_size_to_scale(mut query: Query<(&Size, &mut Transform), Added<Size>>) {
    for (size, mut transform) in &mut query {
        transform.scale = Vec3::splat(size.0 / 2.0);
    }
}
