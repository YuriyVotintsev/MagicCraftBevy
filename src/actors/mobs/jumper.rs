use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use super::super::components::{
    Caster, Collider, DynamicBody, FindNearestEnemy, Health, JumpWalkAnimation, Lifetime,
    OnCollisionDamage, OnCollisionParticles, OnDeathParticles, Projectile, SelfMoving, Shadow,
    ColliderShape, Size, Shape, ShapeKind, Target,
};
use crate::arena::CurrentArenaSize;
use crate::faction::Faction;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, ModifierKind, Stat, StatCalculators};

use super::spawn::{compute_stats, current_max_life, enemy_ability_shape_color, enemy_shape_color};

#[derive(Clone, Deserialize, Debug)]
pub struct JumperStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: f32,
    pub size: f32,
    pub mass: f32,
    pub idle_duration: f32,
    pub jump_duration: f32,
    pub land_duration: f32,
    pub jump_distance: f32,
    pub projectile_count: u32,
    pub projectile_speed: f32,
    pub projectile_size: f32,
    pub projectile_lifetime: f32,
    pub spread_degrees: f32,
}

#[derive(Component)]
pub struct JumperAi {
    pub idle_duration: f32,
    pub jump_duration: f32,
    pub land_duration: f32,
    pub jump_distance: f32,
    pub projectile_count: u32,
    pub projectile_speed: f32,
    pub projectile_size: f32,
    pub projectile_lifetime: f32,
    pub spread_degrees: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum JumperPhase {
    Idle,
    Jump,
    Land,
}

#[derive(Component)]
pub struct JumperAiState {
    phase: JumperPhase,
    elapsed: f32,
    ability_fired: bool,
}

#[derive(Component)]
pub struct RandomJump {
    pub distance: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct RandomJumpState {
    pub elapsed: f32,
    pub duration: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_jumper_ai, jumper_ai_system, init_random_jump, random_jump_system)
            .chain()
            .in_set(GameSet::MobAI),
    );
    app.add_observer(|on: On<Remove, RandomJump>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

pub fn spawn_jumper(
    commands: &mut Commands,
    pos: Vec2,
    s: &JumperStats,
    calculators: &StatCalculators,
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        calculators,
        &[
            (Stat::MovementSpeed, ModifierKind::Flat, s.speed),
            (Stat::MaxLife, ModifierKind::Flat, s.hp),
            (Stat::PhysicalDamage, ModifierKind::Flat, s.damage),
        ],
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
        JumperAi {
            idle_duration: s.idle_duration,
            jump_duration: s.jump_duration,
            land_duration: s.land_duration,
            jump_distance: s.jump_distance,
            projectile_count: s.projectile_count,
            projectile_speed: s.projectile_speed,
            projectile_size: s.projectile_size,
            projectile_lifetime: s.projectile_lifetime,
            spread_degrees: s.spread_degrees,
        },
    )).id();

    commands.entity(id).insert((
        Caster(id),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death_large" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn((
            Shape {
                color: enemy_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 0.5, half_length: 0.5,
            },
            JumpWalkAnimation { bounce_height: 0.7, bounce_duration: 0.5, land_squish: 0.7, land_duration: 0.4 },
        ));
    });

    id
}

fn init_jumper_ai(
    mut commands: Commands,
    query: Query<Entity, Added<JumperAi>>,
) {
    for entity in &query {
        commands.entity(entity).insert(JumperAiState {
            phase: JumperPhase::Idle,
            elapsed: 0.0,
            ability_fired: false,
        });
    }
}

fn jumper_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    stats_q: Query<&ComputedStats>,
    transforms: Query<&Transform>,
    mut query: Query<(Entity, &JumperAi, &mut JumperAiState, Option<&Target>, &Faction), Without<crate::wave::RiseFromGround>>,
) {
    let dt = time.delta_secs();
    for (entity, ai, mut state, target, faction) in &mut query {
        state.elapsed += dt;
        match state.phase {
            JumperPhase::Idle => {
                if state.elapsed >= ai.idle_duration && target.is_some() {
                    state.phase = JumperPhase::Jump;
                    state.elapsed = 0.0;
                    state.ability_fired = false;
                    commands.entity(entity).insert(RandomJump {
                        distance: ai.jump_distance,
                        duration: ai.jump_duration,
                    });
                }
            }
            JumperPhase::Jump => {
                if state.elapsed >= ai.jump_duration {
                    state.phase = JumperPhase::Land;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<RandomJump>();
                }
            }
            JumperPhase::Land => {
                if !state.ability_fired {
                    state.ability_fired = true;
                    let caster_pos = transforms.get(entity).map(|t| crate::coord::to_2d(t.translation)).unwrap_or(Vec2::ZERO);
                    let caster_stats = stats_q.get(entity).ok();
                    fire_jumper_shot(&mut commands, entity, caster_pos, *faction, ai, caster_stats);
                }
                if state.elapsed >= ai.land_duration {
                    state.phase = JumperPhase::Idle;
                    state.elapsed = 0.0;
                }
            }
        }
    }
}

fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

fn fire_jumper_shot(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: Vec2,
    caster_faction: Faction,
    ai: &JumperAi,
    caster_stats: Option<&ComputedStats>,
) {
    let damage = caster_stats.map(|s| s.apply(Stat::PhysicalDamage, 0.0)).unwrap_or(0.0);
    let count = ai.projectile_count as usize;
    let base_dir = Vec2::X;
    let spread_rad = ai.spread_degrees.to_radians();
    let mut rng = rand::rng();

    for i in 0..count {
        let radial_angle = std::f32::consts::TAU * i as f32 / count as f32;
        let spread = rng.random_range(-spread_rad..spread_rad);
        let direction = rotate_vec2(base_dir, radial_angle + spread);
        let velocity = direction * ai.projectile_speed;

        let ground = crate::coord::ground_pos(caster_pos);
        let proj = commands.spawn((
            Transform::from_translation(ground),
            Visibility::default(),
            caster_faction,
            Caster(caster),
            Projectile,
            Size { value: ai.projectile_size },
            Collider { shape: ColliderShape::Circle, sensor: true },
            Lifetime { remaining: ai.projectile_lifetime },
            RigidBody::Kinematic,
            LockedAxes::ROTATION_LOCKED.lock_translation_y(),
            LinearVelocity(crate::coord::ground_vel(velocity)),
            OnCollisionDamage { amount: damage },
        )).id();

        commands.entity(proj).with_children(|p| {
            p.spawn(Shadow { opacity: 0.45 });
            p.spawn(Shape {
                color: enemy_ability_shape_color(), kind: ShapeKind::Circle,
                position: Vec2::ZERO, elevation: 0.7, half_length: 0.5,
            });
        });
    }
    let _ = OnCollisionParticles { config: "enemy_ability_death" };
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
