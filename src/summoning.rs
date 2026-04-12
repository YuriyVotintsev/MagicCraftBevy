use bevy::prelude::*;

use crate::actors::abilities::AbilitiesBalance;
use crate::actors::mobs::{spawn_mob, MobKind, MobsBalance};
use crate::actors::components::common::health::Health;
use crate::particles::{self, ParticleEmitter, SpawnShape};
use crate::run::PlayerDying;
use crate::schedule::GameSet;
use crate::stats::{StatCalculators, StatId, StatRegistry};
use crate::wave::{WaveEnemy, WavePhase, WaveState};
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
pub struct SummoningCircle {
    phase: SummonPhase,
    elapsed: f32,
    pub circle_size: f32,
    pub kind: MobKind,
    extra_modifiers: Vec<(StatId, f32)>,
    pub emitter: Option<Entity>,
}

impl SummoningCircle {
    pub fn new(kind: MobKind, circle_size: f32, extra_modifiers: Vec<(StatId, f32)>) -> Self {
        Self {
            phase: SummonPhase::CircleGrow,
            elapsed: 0.0,
            circle_size,
            kind,
            extra_modifiers,
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
                (init_rise, animate_rise)
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
    mut query: Query<(Entity, &mut SummoningCircle, &mut Transform)>,
    mut wave_state: ResMut<WaveState>,
    mut emitter_query: Query<&mut ParticleEmitter>,
    mobs_balance: Res<MobsBalance>,
    abilities_balance: Res<AbilitiesBalance>,
    stat_registry: Res<StatRegistry>,
    calculators: Res<StatCalculators>,
) {
    let dt = time.delta_secs();

    for (entity, mut circle, mut transform) in &mut query {
        circle.elapsed += dt;
        let pos = crate::coord::to_2d(transform.translation);

        match circle.phase {
            SummonPhase::CircleGrow => {
                let t = (circle.elapsed / CIRCLE_GROW_DURATION).clamp(0.0, 1.0);
                let eased = t * (2.0 - t);

                transform.scale = Vec3::splat(circle.circle_size * eased);

                if circle.emitter.is_none() {
                    let emitter = particles::start_particles(&mut commands, "summon_grow", pos);
                    circle.emitter = Some(emitter);
                }

                if let Some(emitter_entity) = circle.emitter {
                    if let Ok(mut emitter) = emitter_query.get_mut(emitter_entity) {
                        let radius = circle.circle_size * 0.42 * eased;
                        emitter.shape_override = Some(SpawnShape::Circle(radius));
                    }
                }

                if t >= 1.0 {
                    if let Some(emitter_entity) = circle.emitter.take() {
                        particles::stop_particles(&mut commands, emitter_entity);
                    }

                    let mob = spawn_mob(
                        &mut commands,
                        circle.kind,
                        pos,
                        &mobs_balance,
                        &abilities_balance,
                        &stat_registry,
                        &calculators,
                        &circle.extra_modifiers,
                    );
                    commands.entity(mob).insert((
                        WaveEnemy,
                        DespawnOnExit(WavePhase::Combat),
                    ));

                    wave_state.summoning_count = wave_state.summoning_count.saturating_sub(1);

                    circle.phase = SummonPhase::EnemyRise;
                    circle.elapsed = 0.0;
                }
            }
            SummonPhase::EnemyRise => {
                if circle.elapsed >= RISE_DURATION {
                    circle.phase = SummonPhase::CircleShrink;
                    circle.elapsed = 0.0;
                }
            }
            SummonPhase::CircleShrink => {
                let t = (circle.elapsed / CIRCLE_SHRINK_DURATION).clamp(0.0, 1.0);

                transform.scale = Vec3::splat(circle.circle_size * (1.0 - t));

                if t >= 1.0 {
                    commands.entity(entity).despawn();
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

fn cleanup_summoning_on_death(
    mut commands: Commands,
    query: Query<(Entity, &SummoningCircle)>,
    mut wave_state: ResMut<WaveState>,
) {
    for (entity, circle) in &query {
        if let Some(emitter_entity) = circle.emitter {
            particles::stop_particles(&mut commands, emitter_entity);
        }
        commands.entity(entity).despawn();
        if matches!(circle.phase, SummonPhase::CircleGrow) {
            wave_state.summoning_count = wave_state.summoning_count.saturating_sub(1);
        }
    }
}
