use bevy::prelude::*;

use crate::blueprints::BlueprintId;
use crate::blueprints::components::common::health::Health;
use crate::blueprints::spawn_blueprint_entity;
use crate::blueprints::state::{PendingInitialState, StateTransition};
use crate::particles::{self, ParticleEmitter, SpawnShape};
use crate::run::PlayerDying;
use crate::schedule::GameSet;
use crate::wave::{WavePhase, WaveState};
use crate::Faction;

const CIRCLE_GROW_DURATION: f32 = 0.7;
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
    emitter: Option<Entity>,
}

impl SummoningAnimation {
    pub fn new(circle_entity: Entity, circle_size: f32, blueprint_id: BlueprintId) -> Self {
        Self {
            phase: SummonPhase::CircleGrow,
            elapsed: 0.0,
            circle_entity,
            circle_size,
            blueprint_id,
            emitter: None,
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
}

fn animate_summoning(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut query: Query<(Entity, &mut SummoningAnimation, &Transform)>,
    mut circle_query: Query<&mut Transform, Without<SummoningAnimation>>,
    mut wave_state: ResMut<WaveState>,
    mut emitter_query: Query<&mut ParticleEmitter>,
) {
    let dt = time.delta_secs();

    for (entity, mut anim, shell_transform) in &mut query {
        anim.elapsed += dt;
        let pos = crate::coord::to_2d(shell_transform.translation);

        match anim.phase {
            SummonPhase::CircleGrow => {
                let t = (anim.elapsed / CIRCLE_GROW_DURATION).clamp(0.0, 1.0);
                let eased = t * (2.0 - t);

                if let Ok(mut circle_tf) = circle_query.get_mut(anim.circle_entity) {
                    circle_tf.scale = Vec3::splat(anim.circle_size * eased);
                }

                if anim.emitter.is_none() {
                    let emitter = particles::start_particles(&mut commands, "summon_grow", pos);
                    anim.emitter = Some(emitter);
                }

                if let Some(emitter_entity) = anim.emitter {
                    if let Ok(mut emitter) = emitter_query.get_mut(emitter_entity) {
                        let radius = anim.circle_size * 0.42 * eased;
                        emitter.shape_override = Some(SpawnShape::Circle(radius));
                    }
                }

                if t >= 1.0 {
                    if let Some(emitter_entity) = anim.emitter.take() {
                        particles::stop_particles(&mut commands, emitter_entity);
                    }

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
                    particles::start_particles(&mut commands, "summon_burst", pos);

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
        if let Some(emitter_entity) = anim.emitter {
            particles::stop_particles(&mut commands, emitter_entity);
        }
        if let Ok(mut ec) = commands.get_entity(anim.circle_entity) {
            ec.despawn();
        }
        commands.entity(entity).remove::<SummoningAnimation>();
        if matches!(anim.phase, SummonPhase::CircleGrow) {
            wave_state.summoning_count = wave_state.summoning_count.saturating_sub(1);
        }
    }
}
