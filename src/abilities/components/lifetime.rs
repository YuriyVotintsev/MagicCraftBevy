use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::schedule::GameSet;
use crate::GameState;

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
pub struct Lifetime {
    pub remaining: f32,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let remaining = def.0.evaluate_f32(ctx.stats);
    commands.insert(Lifetime { remaining });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        tick_lifetime
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.remaining -= dt;
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
