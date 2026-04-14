use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::actors::combat::Health;
use crate::actors::components::combat::find_nearest_enemy::FindNearestEnemy;
use crate::actors::components::physics::collider::{Collider, Shape as ColliderShape};
use crate::actors::components::physics::dynamic_body::DynamicBody;
use crate::actors::components::visual::jump_walk_animation::{JumpWalkAnimation, SelfMoving};
use crate::actors::components::visual::shadow::Shadow;
use crate::actors::components::physics::size::Size;
use crate::actors::components::visual::sprite::{Sprite, SpriteShape};
use crate::actors::effects::OnDeathParticles;
use crate::actors::components::combat::melee_attacker::MeleeAttacker;
use crate::actors::SpawnSource;
use crate::faction::Faction;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat, StatCalculators};

use super::{compute_stats, current_max_life, enemy_sprite_color};

const LUNGE_DEFAULT_DURATION: f32 = 0.6;

#[derive(Clone, Deserialize, Debug)]
pub struct SlimeSmallStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub mass: f32,
    pub melee_range: f32,
    pub melee_cooldown: f32,
    pub lunge_duration: f32,
}

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
    s: &SlimeSmallStats,
    calculators: &StatCalculators,
    extra_modifiers: &[(Stat, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        calculators,
        &[(Stat::MovementSpeedFlat, s.speed), (Stat::MaxLifeFlat, s.hp), (Stat::PhysicalDamageFlat, s.damage)],
        extra_modifiers,
    );
    let hp = current_max_life(&computed);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: s.mass },
        Health { current: hp },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        LungeMovement { speed: None, duration: Some(s.lunge_duration), pause_duration: 0.4, distance: None },
        MeleeAttacker::new(s.melee_cooldown, s.melee_range),
    )).id();

    commands.entity(id).insert((
        SpawnSource::from_caster(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn((
            Sprite {
                color: enemy_sprite_color(), shape: SpriteShape::Circle,
                position: Vec2::ZERO, scale: 1.0, elevation: 0.5, half_length: 0.5,
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
            .map(|s| s.get(Stat::MovementSpeed))
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
        &SpawnSource,
    ), Without<crate::wave::summoning::RiseFromGround>>,
    transforms: Query<&Transform, Without<LungeMovement>>,
) {
    let dt = time.delta_secs();

    for (entity, transform, mut velocity, lunge, mut state, source) in &mut query {
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
                    let Some(target_entity) = source.target.entity else {
                        velocity.0 = Vec3::ZERO;
                        continue;
                    };
                    let Ok(target_transform) = transforms.get(target_entity) else {
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
