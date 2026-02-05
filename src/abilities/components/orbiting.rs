use std::f32::consts::PI;
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
    pub radius: ScalarExprRaw,
    pub angular_speed: ScalarExprRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub radius: ScalarExpr,
    pub angular_speed: ScalarExpr,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            radius: self.radius.resolve(stat_registry),
            angular_speed: self.angular_speed.resolve(stat_registry),
        }
    }
}

pub fn required_fields_and_nested(raw: &DefRaw) -> (ProvidedFields, Option<(ProvidedFields, &[EntityDefRaw])>) {
    let fields = raw.radius.required_fields().union(raw.angular_speed.required_fields());
    (fields, None)
}

#[derive(Component)]
pub struct OrbitingMovement {
    pub radius: f32,
    pub angular_speed: f32,
    pub current_angle: f32,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let eval_ctx = ctx.eval_context();
    let radius = def.radius.eval(&eval_ctx);
    let angular_speed = def.angular_speed.eval(&eval_ctx);

    let initial_angle = if ctx.count > 1 {
        2.0 * PI * (ctx.index as f32) / (ctx.count as f32)
    } else {
        0.0
    };

    let offset = Vec2::new(initial_angle.cos(), initial_angle.sin()) * radius;
    let source_pos = ctx.source.position.map(|p| p.extend(0.0)).unwrap_or(Vec3::ZERO);
    let position = source_pos + offset.extend(0.0);

    commands.insert((
        OrbitingMovement {
            radius,
            angular_speed,
            current_angle: initial_angle,
        },
        AttachedTo { owner: ctx.caster },
        Transform::from_translation(position),
    ));
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        update_orbiting_positions
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn update_orbiting_positions(
    time: Res<Time>,
    owner_query: Query<&Transform, Without<OrbitingMovement>>,
    mut orb_query: Query<(&AttachedTo, &mut OrbitingMovement, &mut Transform)>,
) {
    for (attached, mut orbiting, mut transform) in &mut orb_query {
        orbiting.current_angle += orbiting.angular_speed * time.delta_secs();

        if let Ok(owner_transform) = owner_query.get(attached.owner) {
            let offset = Vec2::new(
                orbiting.current_angle.cos() * orbiting.radius,
                orbiting.current_angle.sin() * orbiting.radius,
            );
            transform.translation = owner_transform.translation + offset.extend(0.0);
        }
    }
}
