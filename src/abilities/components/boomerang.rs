use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub max_distance: ScalarExprRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub max_distance: ScalarExpr,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            max_distance: self.max_distance.resolve(stat_registry),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    (raw.max_distance.required_fields(), None)
}

#[derive(Component)]
pub struct BoomerangMovement {
    pub max_distance: f32,
    pub speed: f32,
    pub spawn_position: Vec3,
    pub returning: bool,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let max_distance = def.max_distance.eval(&ctx.eval_context());
    let source_pos = ctx.source.position.map(|p| p.extend(0.0)).unwrap_or(Vec3::ZERO);

    commands.insert((
        BoomerangMovement {
            max_distance,
            speed: 0.0,
            spawn_position: source_pos,
            returning: false,
        },
        AttachedTo { owner: ctx.caster },
    ));
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_boomerang_speed, boomerang_movement_system)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_boomerang_speed(
    mut query: Query<(&LinearVelocity, &mut BoomerangMovement), Added<BoomerangMovement>>,
) {
    for (velocity, mut boomerang) in &mut query {
        boomerang.speed = velocity.0.length();
    }
}

fn boomerang_movement_system(
    mut commands: Commands,
    owner_query: Query<&Transform, Without<BoomerangMovement>>,
    mut query: Query<(Entity, &AttachedTo, &mut BoomerangMovement, &Transform, &mut LinearVelocity)>,
) {
    for (entity, attached, mut boomerang, transform, mut velocity) in &mut query {
        if !boomerang.returning {
            let distance = transform.translation.distance(boomerang.spawn_position);
            if distance >= boomerang.max_distance {
                boomerang.returning = true;
            }
        }

        if boomerang.returning {
            if let Ok(owner_transform) = owner_query.get(attached.owner) {
                let to_owner = owner_transform.translation - transform.translation;
                let distance = to_owner.length();

                if distance < 20.0 {
                    commands.entity(entity).despawn();
                    continue;
                }

                let direction = to_owner.truncate().normalize();
                velocity.0 = direction * boomerang.speed;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
