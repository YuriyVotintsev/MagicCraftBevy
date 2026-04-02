use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::blueprints::components::common::health::Health;
use crate::blueprints::components::common::jump_walk_animation::JumpWalkAnimationState;
use crate::blueprints::SpawnSource;
use crate::particles::{Particle, PlayerDeathParticleConfig};
use crate::player::Player;
use crate::schedule::{GameSet, PostGameSet};
use crate::stats::{DeathEvent, SkipCleanup, death_system};
use crate::wave::WavePhase;
use crate::MovementLocked;

#[derive(Resource, Default)]
pub struct RunState {
    pub elapsed: f32,
    pub attempt: u32,
}

const PLAYER_SHRINK_DURATION: f32 = 0.3;

const LANDING_TIMEOUT: f32 = 0.5;

enum DeathPhase {
    Landing { elapsed: f32 },
    Shrinking {
        elapsed: f32,
        initial_scale: Vec3,
        particle_lifetime: f32,
    },
}

#[derive(Resource)]
pub struct PlayerDying {
    phase: DeathPhase,
}

#[derive(Component)]
struct ShrinkToZero {
    elapsed: f32,
    duration: f32,
    initial_scale: Vec3,
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .add_systems(OnEnter(WavePhase::Combat), init_run)
            .add_systems(
                Update,
                (
                    tick_run,
                    drain_stamina.run_if(not(resource_exists::<PlayerDying>)),
                )
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                PostUpdate,
                check_run_end
                    .after(death_system)
                    .in_set(PostGameSet),
            )
            .add_systems(
                Update,
                (
                    mark_new_shrink_targets,
                    animate_shrink_to_zero,
                    player_death_sequence,
                )
                    .in_set(GameSet::Cleanup)
                    .run_if(resource_exists::<PlayerDying>),
            )
            .add_systems(OnExit(WavePhase::Combat), cleanup_player_dying);
    }
}

fn init_run(mut run_state: ResMut<RunState>) {
    run_state.elapsed = 0.0;
    run_state.attempt += 1;
    info!("Starting run #{}", run_state.attempt);
}

fn tick_run(time: Res<Time>, mut run_state: ResMut<RunState>) {
    run_state.elapsed += time.delta_secs();
}

fn drain_stamina(
    time: Res<Time>,
    mut player_query: Query<&mut Health, With<Player>>,
) {
    for mut health in &mut player_query {
        health.current -= time.delta_secs();
    }
}

fn check_run_end(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    player_query: Query<Entity, With<Player>>,
    combat_entities: Query<
        (Entity, &Transform),
        (
            Or<(With<DespawnOnExit<WavePhase>>, With<SpawnSource>)>,
            Without<Player>,
        ),
    >,
    dying: Option<Res<PlayerDying>>,
) {
    if dying.is_some() {
        for _ in death_events.read() {}
        return;
    }
    for event in death_events.read() {
        if player_query.contains(event.entity) {
            commands.entity(event.entity).insert((
                SkipCleanup,
                MovementLocked,
                LinearVelocity(Vec3::ZERO),
                RigidBody::Kinematic,
            ));
            commands.insert_resource(PlayerDying {
                phase: DeathPhase::Landing { elapsed: 0.0 },
            });
            for (entity, transform) in &combat_entities {
                commands.entity(entity).insert(ShrinkToZero {
                    elapsed: 0.0,
                    duration: 0.5,
                    initial_scale: transform.scale,
                });
            }
        }
    }
}

fn mark_new_shrink_targets(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform),
        (
            Or<(With<DespawnOnExit<WavePhase>>, With<SpawnSource>)>,
            Without<Player>,
            Without<ShrinkToZero>,
        ),
    >,
) {
    for (entity, transform) in &query {
        commands.entity(entity).insert(ShrinkToZero {
            elapsed: 0.0,
            duration: 0.5,
            initial_scale: transform.scale,
        });
    }
}

fn animate_shrink_to_zero(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShrinkToZero, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut shrink, mut transform) in &mut query {
        shrink.elapsed += dt;
        let t = (shrink.elapsed / shrink.duration).clamp(0.0, 1.0);
        transform.scale = shrink.initial_scale * (1.0 - t);
        if t >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn player_death_sequence(
    mut commands: Commands,
    time: Res<Time>,
    mut dying: ResMut<PlayerDying>,
    mut player_query: Query<(Entity, &mut Transform, &Children), With<Player>>,
    anim_state_query: Query<&JumpWalkAnimationState>,
    children_query: Query<&Children>,
    shrink_query: Query<(), With<ShrinkToZero>>,
    config: Res<PlayerDeathParticleConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_phase: ResMut<NextState<WavePhase>>,
) {
    let dt = time.delta_secs();

    if let DeathPhase::Shrinking {
        ref mut elapsed,
        initial_scale,
        particle_lifetime,
    } = dying.phase
    {
        *elapsed += dt;
        if let Ok((_, mut transform, _)) = player_query.single_mut() {
            let t = (*elapsed / PLAYER_SHRINK_DURATION).clamp(0.0, 1.0);
            transform.scale = initial_scale * (1.0 - t);
        }
        if *elapsed >= particle_lifetime && shrink_query.is_empty() {
            next_phase.set(WavePhase::Shop);
        }
        return;
    }

    let DeathPhase::Landing { ref mut elapsed } = dying.phase else {
        return;
    };
    *elapsed += dt;

    let Ok((_player_entity, transform, player_children)) = player_query.single_mut() else {
        next_phase.set(WavePhase::Shop);
        return;
    };

    let timed_out = *elapsed >= LANDING_TIMEOUT;
    let mut landed = false;
    for child in player_children.iter() {
        if let Ok(state) = anim_state_query.get(child) {
            landed = landed || state.landed || state.amplitude < 0.01;
        }
        if let Ok(grandchildren) = children_query.get(child) {
            for grandchild in grandchildren.iter() {
                if let Ok(state) = anim_state_query.get(grandchild) {
                    landed = landed || state.landed || state.amplitude < 0.01;
                }
            }
        }
    }

    if !landed && !timed_out {
        return;
    }

    let pos = crate::coord::to_2d(transform.translation);
    let initial_scale = transform.scale;

    let color = crate::palette::color(&config.color);
    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    });
    let mesh = meshes.add(Sphere::new(0.5));
    let mut rng = rand::rng();
    for _ in 0..config.count {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = config.speed * rng.random_range(0.5..1.0);
        let dir = Vec2::new(angle.cos(), angle.sin());
        let start_scale = config.start_size / 2.0;
        let end_scale = config.end_size / 2.0;
        let spawn_pos = crate::coord::ground_pos(pos);
        commands.spawn((
            Particle {
                velocity: crate::coord::ground_vel(dir * speed),
                remaining: config.lifetime,
                lifetime: config.lifetime,
                start_scale,
                end_scale,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(Vec3::new(spawn_pos.x, config.elevation, spawn_pos.z))
                .with_scale(Vec3::splat(start_scale)),
        ));
    }

    let _ = player_children;
    let _ = transform;

    dying.phase = DeathPhase::Shrinking {
        elapsed: 0.0,
        initial_scale,
        particle_lifetime: config.lifetime,
    };
}

fn cleanup_player_dying(mut commands: Commands) {
    commands.remove_resource::<PlayerDying>();
}
