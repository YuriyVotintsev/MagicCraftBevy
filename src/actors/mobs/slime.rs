use avian3d::prelude::*;
use bevy::prelude::*;

use crate::balance::MobCommonStats;
use super::super::components::{
    JumpWalkAnimation, MeleeAttacker, SelfMoving, Shape, ShapeKind,
};
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, ModifierKind, Stat, StatCalculators};

use super::spawn::{enemy_shape_color, spawn_enemy_core, EnemyBody, WaveModifiers};

const LUNGE_DEFAULT_DURATION: f32 = 0.6;

const SLIME_LUNGE_DURATION: f32 = 0.5;

#[derive(Component)]
pub struct LungeMovement {
    pub speed: Option<f32>,
    pub duration: Option<f32>,
    pub pause_duration: f32,
    pub distance: Option<f32>,
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

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_lunge_movement, lunge_movement_system)
            .chain()
            .in_set(GameSet::MobAI),
    );
    app.add_observer(|on: On<Remove, LungeMovement>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

pub fn spawn_slime_small(
    commands: &mut Commands,
    pos: Vec2,
    s: &MobCommonStats,
    calculators: &StatCalculators,
    wave_mods: WaveModifiers,
) -> Entity {
    let speed = s.speed.unwrap_or(0.0);
    let mass = s.mass.unwrap_or(1.0);
    let id = spawn_enemy_core(
        commands,
        pos,
        calculators,
        &[
            (Stat::MovementSpeed, ModifierKind::Flat, speed),
            (Stat::MaxLife, ModifierKind::Flat, s.hp),
            (Stat::PhysicalDamage, ModifierKind::Flat, s.damage),
        ],
        s.size,
        EnemyBody::Dynamic { mass },
        "enemy_death",
        wave_mods,
    );

    commands.entity(id).insert((
        LungeMovement { speed: None, duration: Some(SLIME_LUNGE_DURATION), pause_duration: 0.4, distance: None },
        MeleeAttacker::new(s.attack_speed.unwrap_or(0.5)),
    ));

    commands.entity(id).with_children(|p| {
        p.spawn((
            Shape {
                color: enemy_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.7, bounce_duration: 0.5, land_squish: 0.3, land_duration: 0.4 },
        ));
    });

    id
}

fn init_lunge_movement(
    mut commands: Commands,
    query: Query<(Entity, &LungeMovement, Option<&ComputedStats>), Added<LungeMovement>>,
) {
    for (entity, lunge, stats) in &query {
        let stat_speed = stats
            .map(|s| s.final_of(Stat::MovementSpeed))
            .filter(|v| *v > 0.0)
            .unwrap_or(400.0);

        let (speed, duration) = match (lunge.speed, lunge.duration, lunge.distance) {
            (Some(s), _, Some(d)) => (s, d / s),
            (None, Some(dur), Some(d)) => (d / dur, dur),
            (None, None, Some(d)) => (stat_speed, d / stat_speed),
            (Some(s), dur, None) => (s, dur.unwrap_or(LUNGE_DEFAULT_DURATION)),
            (None, dur, None) => (stat_speed, dur.unwrap_or(LUNGE_DEFAULT_DURATION)),
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
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Transform,
        &mut LinearVelocity,
        &LungeMovement,
        &mut LungeMovementState,
    ), Without<crate::wave::RiseFromGround>>,
    player: Option<Single<&Transform, (With<crate::actors::Player>, Without<LungeMovement>)>>,
) {
    let dt = time.delta_secs();

    for (entity, transform, mut velocity, lunge, mut state) in &mut query {
        state.elapsed += dt;

        match state.phase {
            LungePhase::Lunging => {
                if state.elapsed >= state.duration {
                    state.phase = LungePhase::Pausing;
                    state.elapsed = 0.0;
                    velocity.0 = Vec3::ZERO;
                    commands.entity(entity).remove::<SelfMoving>();
                    continue;
                }

                if state.direction == Vec2::ZERO {
                    let Some(ref player) = player else {
                        velocity.0 = Vec3::ZERO;
                        continue;
                    };
                    let diff = crate::coord::to_2d(player.translation - transform.translation);
                    if diff.length_squared() > 1.0 {
                        state.direction = diff.normalize();
                    } else {
                        velocity.0 = Vec3::ZERO;
                        continue;
                    }
                }

                commands.entity(entity).insert(SelfMoving);
                velocity.0 = crate::coord::ground_vel(state.direction * state.speed);
            }
            LungePhase::Pausing => {
                velocity.0 = Vec3::ZERO;
                commands.entity(entity).remove::<SelfMoving>();
                if state.elapsed >= lunge.pause_duration {
                    state.phase = LungePhase::Lunging;
                    state.elapsed = 0.0;
                    state.direction = Vec2::ZERO;
                }
            }
        }
    }
}
