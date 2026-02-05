use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::schedule::GameSet;
use crate::GameState;

use super::lifetime::Lifetime;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub start_size: ScalarExprRaw,
    pub end_size: ScalarExprRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub start_size: ScalarExpr,
    pub end_size: ScalarExpr,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            start_size: self.start_size.resolve(stat_registry),
            end_size: self.end_size.resolve(stat_registry),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let fields = raw.start_size.required_fields().union(raw.end_size.required_fields());
    (fields, None)
}

#[derive(Component)]
pub struct Growing {
    pub start_size: f32,
    pub end_size: f32,
    pub duration: f32,
    pub elapsed: f32,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let start_size = def.start_size.eval(&eval_ctx);
    let end_size = def.end_size.eval(&eval_ctx);
    commands.insert(Growing {
        start_size,
        end_size,
        duration: 0.0,
        elapsed: 0.0,
    });
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
    mut query: Query<(&mut Growing, &Lifetime), Changed<Lifetime>>,
) {
    for (mut growing, lifetime) in &mut query {
        if growing.duration == 0.0 {
            growing.duration = lifetime.remaining;
        }
    }
}
