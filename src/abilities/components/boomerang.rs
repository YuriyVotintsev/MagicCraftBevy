use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::context::ProvidedFields;
use crate::abilities::entity_def::EntityDefRaw;
use crate::abilities::expr::{ScalarExpr, ScalarExprRaw, VecExpr, VecExprRaw};
use crate::abilities::spawn::SpawnContext;
use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

use super::speed::Speed;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub max_distance: ScalarExprRaw,
    #[serde(default)]
    pub direction: Option<VecExprRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub max_distance: ScalarExpr,
    pub direction: Option<VecExpr>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            max_distance: self.max_distance.resolve(stat_registry),
            direction: self.direction.as_ref().map(|d| d.resolve(stat_registry)),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let mut fields = raw.max_distance.required_fields();
    match &raw.direction {
        Some(dir) => fields = fields.union(dir.required_fields()),
        None => fields = fields.union(ProvidedFields::TARGET_DIRECTION),
    }
    (fields, None)
}

#[derive(Component)]
pub struct Boomerang {
    pub max_distance: f32,
    pub direction: Vec2,
    pub spawn_position: Vec3,
    pub returning: bool,
}

pub fn insert_component(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let max_distance = def.max_distance.eval(&eval_ctx);

    let direction = match &def.direction {
        Some(dir_expr) => dir_expr.eval(&eval_ctx).normalize_or_zero(),
        None => ctx.target.direction.expect("Boomerang requires target.direction when direction field is not specified"),
    };

    let spawn_position = ctx.source.position.map(|p| p.extend(0.0)).unwrap_or(Vec3::ZERO);

    commands.insert(Boomerang {
        max_distance,
        direction,
        spawn_position,
        returning: false,
    });
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
        commands.entity(entity).insert((
            AttachedTo { owner: source.caster },
            RigidBody::Kinematic,
            LinearVelocity(boomerang.direction * speed.0),
        ));
    }
}

fn boomerang_system(
    mut commands: Commands,
    owner_query: Query<&Transform, Without<Boomerang>>,
    mut query: Query<(Entity, &AttachedTo, &Speed, &mut Boomerang, &Transform, &mut LinearVelocity)>,
) {
    for (entity, attached, speed, mut boomerang, transform, mut velocity) in &mut query {
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
                velocity.0 = direction * speed.0;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
