use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

use super::speed::Speed;

#[ability_component]
pub struct Boomerang {
    pub max_distance: ScalarExpr,
    #[default_expr("target.direction")]
    pub direction: VecExpr,
    #[default_expr("source.position")]
    pub spawn_position: VecExpr,
}

#[derive(Component, Default)]
pub struct BoomerangState {
    pub returning: bool,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_boomerang, boomerang_system)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_boomerang(
    mut commands: Commands,
    query: Query<(Entity, &Speed, &Boomerang, &crate::abilities::AbilitySource), Added<Boomerang>>,
) {
    for (entity, speed, boomerang, source) in &query {
        let direction = boomerang.direction.normalize_or_zero();
        commands.entity(entity).insert((
            AttachedTo { owner: source.caster },
            RigidBody::Kinematic,
            LinearVelocity(direction * speed.value),
            BoomerangState::default(),
            Transform::from_translation(boomerang.spawn_position.extend(0.0)),
        ));
    }
}

fn boomerang_system(
    mut commands: Commands,
    owner_query: Query<&Transform, Without<Boomerang>>,
    mut query: Query<(Entity, &AttachedTo, &Speed, &Boomerang, &mut BoomerangState, &Transform, &mut LinearVelocity)>,
) {
    for (entity, attached, speed, boomerang, mut state, transform, mut velocity) in &mut query {
        if !state.returning {
            let distance = transform.translation.truncate().distance(boomerang.spawn_position);
            if distance >= boomerang.max_distance {
                state.returning = true;
            }
        }

        if state.returning {
            if let Ok(owner_transform) = owner_query.get(attached.owner) {
                let to_owner = owner_transform.translation - transform.translation;
                let distance = to_owner.length();

                if distance < 20.0 {
                    commands.entity(entity).despawn();
                    continue;
                }

                let direction = to_owner.truncate().normalize();
                velocity.0 = direction * speed.value;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
