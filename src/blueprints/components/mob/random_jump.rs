use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;
use rand::Rng;

use crate::balance::CurrentArenaSize;
use crate::movement::SelfMoving;

#[blueprint_component]
pub struct RandomJump {
    #[raw(default = 350.0)]
    pub distance: ScalarExpr,
    #[raw(default = 0.5)]
    pub duration: ScalarExpr,
}

#[derive(Component)]
pub struct RandomJumpState {
    pub elapsed: f32,
    pub duration: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_random_jump, random_jump_system)
            .chain()
            .in_set(crate::schedule::GameSet::MobAI),
    );
    app.add_observer(|on: On<Remove, RandomJump>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

fn init_random_jump(
    mut commands: Commands,
    query: Query<(Entity, &RandomJump, &Transform), Added<RandomJump>>,
    arena_size: Res<CurrentArenaSize>,
) {
    let margin = 120.0;
    let hw = arena_size.half_w() - margin;
    let hh = arena_size.half_h() - margin;
    let mut rng = rand::rng();

    for (entity, jump, transform) in &query {
        let current = crate::coord::to_2d(transform.translation);

        let direction = {
            let mut dir = Vec2::ZERO;
            for _ in 0..32 {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let candidate = Vec2::new(angle.cos(), angle.sin());
                let landing = current + candidate * jump.distance;
                if landing.x.abs() <= hw && landing.y.abs() <= hh {
                    dir = candidate;
                    break;
                }
            }
            if dir == Vec2::ZERO {
                Vec2::ZERO
            } else {
                dir
            }
        };

        let speed = if jump.duration > 0.0 {
            jump.distance / jump.duration
        } else {
            0.0
        };

        commands.entity(entity).insert((
            RandomJumpState {
                elapsed: 0.0,
                duration: jump.duration,
            },
            LinearVelocity(crate::coord::ground_vel(direction * speed)),
            SelfMoving,
        ));
    }
}

fn random_jump_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LinearVelocity, &mut RandomJumpState)>,
) {
    let dt = time.delta_secs();
    for (entity, mut velocity, mut state) in &mut query {
        state.elapsed += dt;
        if state.elapsed >= state.duration {
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<SelfMoving>();
        }
    }
}
