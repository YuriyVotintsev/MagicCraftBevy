use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::stats::{ComputedStats, StatRegistry};

#[blueprint_component]
pub struct LungeMovement {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
    pub speed: Option<ScalarExpr>,
    pub lunge_duration: Option<ScalarExpr>,
    #[raw(default = 0.4)]
    pub pause_duration: ScalarExpr,
    pub distance: Option<ScalarExpr>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LungePhase {
    Lunging,
    Pausing,
}

#[derive(Component)]
pub struct LungeMovementState {
    pub phase: LungePhase,
    pub elapsed: f32,
    direction: Vec2,
    pub speed: f32,
    pub duration: f32,
}

const DEFAULT_DURATION: f32 = 0.6;
const LUNGE_INTEGRAL: f32 = 0.33;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_lunge_movement, lunge_movement_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
    app.add_observer(|on: On<Remove, LungeMovement>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

fn init_lunge_movement(
    mut commands: Commands,
    query: Query<(Entity, &LungeMovement, Option<&ComputedStats>), Added<LungeMovement>>,
    stat_registry: Option<Res<StatRegistry>>,
) {
    for (entity, lunge, stats) in &query {
        let stat_speed = stats
            .and_then(|s| {
                stat_registry
                    .as_ref()
                    .and_then(|sr| sr.get("movement_speed"))
                    .map(|id| s.get(id))
            })
            .unwrap_or(400.0);

        let (speed, duration) = match (lunge.speed, lunge.lunge_duration, lunge.distance) {
            (Some(s), _, Some(d)) => (s, d / (s * LUNGE_INTEGRAL)),
            (None, Some(dur), Some(d)) => (d / (dur * LUNGE_INTEGRAL), dur),
            (None, None, Some(d)) => (stat_speed, d / (stat_speed * LUNGE_INTEGRAL)),
            (Some(s), dur, None) => (s, dur.unwrap_or(DEFAULT_DURATION)),
            (None, dur, None) => (stat_speed, dur.unwrap_or(DEFAULT_DURATION)),
        };

        commands.entity(entity).insert(LungeMovementState {
            phase: LungePhase::Lunging,
            elapsed: 0.0,
            direction: Vec2::ZERO,
            speed,
            duration,
        });
    }
}

fn lunge_movement_system(
    time: Res<Time>,
    mut query: Query<(
        &Transform,
        &mut LinearVelocity,
        &LungeMovement,
        &mut LungeMovementState,
    )>,
    transforms: Query<&Transform, Without<LungeMovement>>,
) {
    let dt = time.delta_secs();

    for (transform, mut velocity, lunge, mut state) in &mut query {
        state.elapsed += dt;

        match state.phase {
            LungePhase::Lunging => {
                if state.elapsed >= state.duration {
                    state.phase = LungePhase::Pausing;
                    state.elapsed = 0.0;
                    velocity.0 = Vec3::ZERO;
                    continue;
                }

                if state.direction == Vec2::ZERO {
                    let Ok(target_transform) = transforms.get(lunge.target) else {
                        velocity.0 = Vec3::ZERO;
                        continue;
                    };
                    let diff = crate::coord::to_2d(target_transform.translation - transform.translation);
                    if diff.length_squared() > 1.0 {
                        state.direction = diff.normalize();
                    } else {
                        velocity.0 = Vec3::ZERO;
                        continue;
                    }
                }

                let t = state.elapsed / state.duration;
                let factor = (std::f32::consts::PI * t * t).sin();
                velocity.0 = crate::coord::ground_vel(state.direction * state.speed * factor);
            }
            LungePhase::Pausing => {
                velocity.0 = Vec3::ZERO;
                if state.elapsed >= lunge.pause_duration {
                    state.phase = LungePhase::Lunging;
                    state.elapsed = 0.0;
                    state.direction = Vec2::ZERO;
                }
            }
        }
    }
}
