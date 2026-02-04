use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::schedule::GameSet;
use crate::GameState;

use super::lifetime::Lifetime;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub start_size: ParamValueRaw,
    pub end_size: ParamValueRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub start_size: ParamValue,
    pub end_size: ParamValue,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            start_size: resolve_param_value(&self.start_size, stat_registry),
            end_size: resolve_param_value(&self.end_size, stat_registry),
        }
    }
}

#[derive(Component)]
pub struct Growing {
    pub start_size: f32,
    pub end_size: f32,
    pub duration: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct GrowingParams {
    pub start_size: f32,
    pub end_size: f32,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let start_size = def.start_size.evaluate_f32(ctx.stats);
    let end_size = def.end_size.evaluate_f32(ctx.stats);
    commands.insert(Growing {
        start_size,
        end_size,
        duration: 0.0,
        elapsed: 0.0,
    });
    commands.insert(GrowingParams { start_size, end_size });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        tick_growing
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(PostUpdate, sync_growing_with_lifetime);
}

fn tick_growing(time: Res<Time>, mut query: Query<(&mut Growing, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut growing, mut transform) in &mut query {
        if growing.duration <= 0.0 {
            transform.scale = Vec3::splat(growing.start_size / 2.0);
            continue;
        }
        growing.elapsed += dt;
        let t = (growing.elapsed / growing.duration).clamp(0.0, 1.0);
        let size = growing.start_size + (growing.end_size - growing.start_size) * t;
        transform.scale = Vec3::splat(size / 2.0);
    }
}

fn sync_growing_with_lifetime(
    mut query: Query<(&mut Growing, &GrowingParams, &Lifetime), Changed<Lifetime>>,
) {
    for (mut growing, params, lifetime) in &mut query {
        if growing.duration == 0.0 {
            growing.start_size = params.start_size;
            growing.end_size = params.end_size;
            growing.duration = lifetime.remaining;
        }
    }
}
