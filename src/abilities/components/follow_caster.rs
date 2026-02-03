use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::spawn::SpawnContext;
use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DefRaw {}

#[derive(Debug, Clone)]
pub struct Def;

impl DefRaw {
    pub fn resolve(&self, _stat_registry: &crate::stats::StatRegistry) -> Def {
        Def
    }
}

#[derive(Component)]
pub struct FollowCaster;

pub fn spawn(commands: &mut EntityCommands, _def: &Def, ctx: &SpawnContext) {
    let source_pos = ctx.source.as_point().unwrap_or(Vec3::ZERO);
    commands.insert((
        FollowCaster,
        AttachedTo { owner: ctx.caster },
        Transform::from_translation(source_pos),
    ));
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        follow_caster_system
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn follow_caster_system(
    owner_query: Query<&Transform, Without<FollowCaster>>,
    mut follower_query: Query<(&AttachedTo, &mut Transform), With<FollowCaster>>,
) {
    for (attached, mut transform) in &mut follower_query {
        if let Ok(owner_transform) = owner_query.get(attached.owner) {
            transform.translation = owner_transform.translation;
        }
    }
}
