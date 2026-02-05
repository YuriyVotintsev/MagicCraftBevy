use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

#[ability_component]
pub struct FollowCaster;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_follow_caster, follow_caster_system)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_follow_caster(
    mut commands: Commands,
    query: Query<(Entity, &crate::abilities::AbilitySource), Added<FollowCaster>>,
) {
    for (entity, source) in &query {
        commands.entity(entity).insert((
            AttachedTo { owner: source.caster },
            Transform::default(),
        ));
    }
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
