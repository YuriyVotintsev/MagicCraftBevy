use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;

use super::fan::Fan;
use super::parallel::Parallel;
use super::radial::Radial;

#[blueprint_component]
pub struct Boomerang {
    pub speed: ScalarExpr,
    pub max_distance: ScalarExpr,
    #[default_expr("target.direction")]
    pub direction: VecExpr,
    #[default_expr("source.position")]
    pub spawn_position: VecExpr,
    #[default_expr("caster.entity")]
    pub return_to: EntityExpr,
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
            .in_set(GameSet::BlueprintExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn rotate_vec2(v: Vec2, angle_rad: f32) -> Vec2 {
    let (sin, cos) = angle_rad.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn init_boomerang(
    mut commands: Commands,
    query: Query<
        (Entity, &Boomerang, &SpawnSource, Option<&Parallel>, Option<&Fan>, Option<&Radial>),
        Added<Boomerang>,
    >,
) {
    for (entity, boomerang, source, parallel, fan, radial) in &query {
        let base_direction = boomerang.direction.normalize_or_zero();
        let mut direction = base_direction;
        let mut position = boomerang.spawn_position;

        if let Some(parallel) = parallel {
            let offset = parallel.gap * (source.index as f32 - (source.count as f32 - 1.0) / 2.0);
            let perpendicular = Vec2::new(-base_direction.y, base_direction.x);
            position += perpendicular * offset;
        } else if let Some(fan) = fan {
            if source.count > 1 {
                let t = source.index as f32 / (source.count as f32 - 1.0) - 0.5;
                direction = rotate_vec2(base_direction, (fan.angle * t).to_radians());
            }
        } else if radial.is_some() {
            let angle = std::f32::consts::TAU * source.index as f32 / source.count as f32;
            direction = rotate_vec2(base_direction, angle);
        }

        commands.entity(entity).insert((
            AttachedTo { owner: boomerang.return_to },
            RigidBody::Kinematic,
            LinearVelocity(direction * boomerang.speed),
            BoomerangState::default(),
            Transform::from_translation(position.extend(0.0)),
        ));
    }
}

fn boomerang_system(
    mut commands: Commands,
    owner_query: Query<&Transform, Without<Boomerang>>,
    mut query: Query<(Entity, &AttachedTo, &Boomerang, &mut BoomerangState, &Transform, &mut LinearVelocity)>,
) {
    for (entity, attached, boomerang, mut state, transform, mut velocity) in &mut query {
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
                velocity.0 = direction * boomerang.speed;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
