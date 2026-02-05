use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::schedule::GameSet;
use crate::GameState;

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
pub struct Lifetime {
    pub remaining: f32,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let remaining = def.0.eval(&ctx.eval_context());
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
