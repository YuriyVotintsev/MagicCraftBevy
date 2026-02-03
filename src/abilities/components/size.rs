use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw(pub ParamValueRaw);

#[derive(Debug, Clone)]
pub struct Def(pub ParamValue);

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def(resolve_param_value(&self.0, stat_registry))
    }
}

#[derive(Component)]
pub struct Size(pub f32);

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let size = def.0.evaluate_f32(ctx.stats);
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
