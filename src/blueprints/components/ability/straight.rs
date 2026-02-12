use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use rand::Rng;

use crate::blueprints::SpawnSource;
use crate::schedule::GameSet;
use crate::GameState;

use super::fan::Fan;
use super::parallel::Parallel;
use super::radial::Radial;

#[blueprint_component]
pub struct Straight {
    pub speed: ScalarExpr,
    #[raw(default = 0)]
    pub spread: ScalarExpr,
    #[default_expr("target.direction")]
    pub direction: VecExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        init_straight
            .in_set(GameSet::BlueprintExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn rotate_vec2(v: Vec2, angle_rad: f32) -> Vec2 {
    let (sin, cos) = angle_rad.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn init_straight(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Straight,
            &SpawnSource,
            &mut Transform,
            Option<&Parallel>,
            Option<&Fan>,
            Option<&Radial>,
        ),
        Added<Straight>,
    >,
) {
    for (entity, straight, source, mut transform, parallel, fan, radial) in &mut query {
        let base_direction = straight.direction.normalize_or_zero();
        let mut direction = base_direction;

        if let Some(parallel) = parallel {
            let offset = parallel.gap * (source.index as f32 - (source.count as f32 - 1.0) / 2.0);
            let perpendicular = Vec2::new(-base_direction.y, base_direction.x);
            transform.translation += (perpendicular * offset).extend(0.0);
        } else if let Some(fan) = fan {
            if source.count > 1 {
                let t = source.index as f32 / (source.count as f32 - 1.0) - 0.5;
                direction = rotate_vec2(base_direction, (fan.angle * t).to_radians());
            }
        } else if radial.is_some() {
            let angle = std::f32::consts::TAU * source.index as f32 / source.count as f32;
            direction = rotate_vec2(base_direction, angle);
        }

        if straight.spread > 0.0 {
            let spread_rad = straight.spread.to_radians();
            let angle_offset = rand::rng().random_range(-spread_rad..spread_rad);
            direction = rotate_vec2(direction, angle_offset);
        }

        commands.entity(entity).insert((
            RigidBody::Kinematic,
            LinearVelocity(direction * straight.speed),
        ));
    }
}
