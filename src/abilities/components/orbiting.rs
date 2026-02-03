use std::f32::consts::PI;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::common::AttachedTo;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::StatRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub radius: ParamValueRaw,
    pub angular_speed: ParamValueRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub radius: ParamValue,
    pub angular_speed: ParamValue,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> Def {
        Def {
            radius: resolve_param_value(&self.radius, stat_registry),
            angular_speed: resolve_param_value(&self.angular_speed, stat_registry),
        }
    }
}

#[derive(Component)]
pub struct OrbitingMovement {
    pub radius: f32,
    pub angular_speed: f32,
    pub current_angle: f32,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let radius = def.radius.evaluate_f32(ctx.stats);
    let angular_speed = def.angular_speed.evaluate_f32(ctx.stats);

    let initial_angle = if ctx.count > 1 {
        2.0 * PI * (ctx.index as f32) / (ctx.count as f32)
    } else {
        0.0
    };

    let offset = Vec2::new(initial_angle.cos(), initial_angle.sin()) * radius;
    let source_pos = ctx.source.as_point().unwrap_or(Vec3::ZERO);
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
