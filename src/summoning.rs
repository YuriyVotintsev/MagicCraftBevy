use bevy::prelude::*;
use rand::Rng;

use crate::blueprints::BlueprintId;
use crate::blueprints::components::common::health::Health;
use crate::blueprints::spawn_blueprint_entity;
use crate::blueprints::state::{PendingInitialState, StateTransition};
use crate::particles::Particle;
use crate::run::PlayerDying;
use crate::schedule::GameSet;
use crate::wave::{WavePhase, WaveState};
use crate::Faction;

const CIRCLE_GROW_DURATION: f32 = 0.4;
const CIRCLE_SHRINK_DURATION: f32 = 0.3;
const RISE_DURATION: f32 = 0.4;
const RISE_Y_OFFSET: f32 = 200.0;
pub const DEFAULT_CIRCLE_SIZE: f32 = 150.0;

enum SummonPhase {
    CircleGrow,
    EnemyRise,
    CircleShrink,
}

#[derive(Component)]
pub struct SummoningAnimation {
    phase: SummonPhase,
    elapsed: f32,
    pub circle_entity: Entity,
    pub circle_size: f32,
    pub blueprint_id: BlueprintId,
}

impl SummoningAnimation {
    pub fn new(circle_entity: Entity, circle_size: f32, blueprint_id: BlueprintId) -> Self {
        Self {
            phase: SummonPhase::CircleGrow,
            elapsed: 0.0,
            circle_entity,
            circle_size,
            blueprint_id,
        }
    }
}

#[derive(Component)]
pub struct RiseFromGround {
    elapsed: f32,
    target_y: f32,
}

#[derive(Resource)]
pub struct SummoningCircleMesh(pub Handle<Mesh>);

#[derive(Resource)]
pub struct SummoningCircleMaterial(pub Handle<StandardMaterial>);

#[derive(Resource)]
struct SummonParticleMesh(Handle<Mesh>);

#[derive(Resource)]
struct SummonParticleMaterial(Handle<StandardMaterial>);

pub struct SummoningPlugin;

impl Plugin for SummoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_resources)
            .add_systems(
                Update,
                animate_summoning
                    .in_set(GameSet::Spawning)
                    .run_if(in_state(WavePhase::Combat))
                    .run_if(not(resource_exists::<PlayerDying>)),
            )
            .add_systems(
                Last,
                (init_rise, animate_rise, activate_pending_initial_state)
                    .chain()
                    .run_if(in_state(WavePhase::Combat)),
            )
            .add_systems(
                Update,
                cleanup_summoning_on_death
                    .in_set(GameSet::Cleanup)
                    .run_if(resource_exists::<PlayerDying>),
            );
    }
}

fn setup_resources(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(SummoningCircleMesh(
        meshes.add(Annulus::new(0.35, 0.5)),
    ));
    commands.insert_resource(SummoningCircleMaterial(
        materials.add(StandardMaterial {
            base_color: crate::palette::color_alpha("enemy", 0.7),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    ));
    commands.insert_resource(SummonParticleMesh(meshes.add(Sphere::new(0.5))));
    commands.insert_resource(SummonParticleMaterial(
        materials.add(StandardMaterial {
            base_color: crate::palette::color("enemy"),
            unlit: true,
            ..default()
        }),
    ));
}

fn animate_summoning(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut SummoningAnimation, &Transform)>,
    mut circle_query: Query<&mut Transform, Without<SummoningAnimation>>,
    mut wave_state: ResMut<WaveState>,
    particle_mesh: Res<SummonParticleMesh>,
    particle_material: Res<SummonParticleMaterial>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (entity, mut anim, shell_transform) in &mut query {
        anim.elapsed += dt;
        let pos = shell_transform.translation;

        match anim.phase {
            SummonPhase::CircleGrow => {
                let t = (anim.elapsed / CIRCLE_GROW_DURATION).clamp(0.0, 1.0);
                let eased = t * (2.0 - t);

                if let Ok(mut circle_tf) = circle_query.get_mut(anim.circle_entity) {
                    circle_tf.scale = Vec3::splat(anim.circle_size * eased);
                }

                let particle_count = rng.random_range(1..=2u32);
                for _ in 0..particle_count {
                    let angle = rng.random_range(0.0..std::f32::consts::TAU);
                    let radius = anim.circle_size * 0.42 * eased;
                    let offset = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
                    let start_scale = 8.0;
                    commands.spawn((
                        Particle {
                            velocity: Vec3::new(0.0, 100.0, 0.0),
                            remaining: 0.4,
                            lifetime: 0.4,
                            start_scale,
                            end_scale: 0.0,
                        },
                        Mesh3d(particle_mesh.0.clone()),
                        MeshMaterial3d(particle_material.0.clone()),
                        Transform::from_translation(pos + offset)
                            .with_scale(Vec3::splat(start_scale)),
                    ));
                }

                if t >= 1.0 {
                    spawn_blueprint_entity(
                        &mut commands,
                        entity,
                        Faction::Enemy,
                        anim.blueprint_id,
                        true,
                    );
                    wave_state.summoning_count = wave_state.summoning_count.saturating_sub(1);

                    anim.phase = SummonPhase::EnemyRise;
                    anim.elapsed = 0.0;
                }
            }
            SummonPhase::EnemyRise => {
                if anim.elapsed >= RISE_DURATION {
                    for _ in 0..rng.random_range(6..=8u32) {
                        let angle = rng.random_range(0.0..std::f32::consts::TAU);
                        let dir = Vec2::new(angle.cos(), angle.sin());
                        let speed = rng.random_range(80.0..160.0);
                        let start_scale = 10.0;
                        commands.spawn((
                            Particle {
                                velocity: Vec3::new(dir.x * speed, 120.0, dir.y * speed),
                                remaining: 0.5,
                                lifetime: 0.5,
                                start_scale,
                                end_scale: 0.0,
                            },
                            Mesh3d(particle_mesh.0.clone()),
                            MeshMaterial3d(particle_material.0.clone()),
                            Transform::from_translation(pos)
                                .with_scale(Vec3::splat(start_scale)),
                        ));
                    }

                    anim.phase = SummonPhase::CircleShrink;
                    anim.elapsed = 0.0;
                }
            }
            SummonPhase::CircleShrink => {
                let t = (anim.elapsed / CIRCLE_SHRINK_DURATION).clamp(0.0, 1.0);

                if let Ok(mut circle_tf) = circle_query.get_mut(anim.circle_entity) {
                    circle_tf.scale = Vec3::splat(anim.circle_size * (1.0 - t));
                }

                if t >= 1.0 {
                    if let Ok(mut ec) = commands.get_entity(anim.circle_entity) {
                        ec.despawn();
                    }
                    commands.entity(entity).remove::<SummoningAnimation>();
                }
            }
        }
    }
}

fn init_rise(
    mut commands: Commands,
    mut query: Query<
        (Entity, &mut Transform, &Faction),
        Added<Health>,
    >,
) {
    for (entity, mut transform, faction) in &mut query {
        if *faction != Faction::Enemy {
            continue;
        }
        let target_y = transform.translation.y;
        transform.translation.y = target_y - RISE_Y_OFFSET;
        commands.entity(entity).insert(RiseFromGround {
            elapsed: 0.0,
            target_y,
        });
    }
}

fn animate_rise(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut RiseFromGround, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut rise, mut transform) in &mut query {
        rise.elapsed += dt;
        let t = (rise.elapsed / RISE_DURATION).clamp(0.0, 1.0);
        let eased = t * (2.0 - t);
        transform.translation.y = (rise.target_y - RISE_Y_OFFSET) + RISE_Y_OFFSET * eased;
        if t >= 1.0 {
            transform.translation.y = rise.target_y;
            commands.entity(entity).remove::<RiseFromGround>();
        }
    }
}

fn activate_pending_initial_state(
    mut commands: Commands,
    query: Query<(Entity, &PendingInitialState), Without<RiseFromGround>>,
    mut events: MessageWriter<StateTransition>,
) {
    for (entity, pending) in &query {
        events.write(StateTransition {
            entity,
            to: pending.0,
        });
        commands.entity(entity).remove::<PendingInitialState>();
    }
}

fn cleanup_summoning_on_death(
    mut commands: Commands,
    query: Query<(Entity, &SummoningAnimation)>,
    mut wave_state: ResMut<WaveState>,
) {
    for (entity, anim) in &query {
        if let Ok(mut ec) = commands.get_entity(anim.circle_entity) {
            ec.despawn();
        }
        commands.entity(entity).remove::<SummoningAnimation>();
        if matches!(anim.phase, SummonPhase::CircleGrow) {
            wave_state.summoning_count = wave_state.summoning_count.saturating_sub(1);
        }
    }
}
