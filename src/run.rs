use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::components::common::health::Health;
use crate::actors::components::common::jump_walk_animation::JumpWalkAnimationState;
use crate::actors::SpawnSource;
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
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
        particle_lifetime: f32,
    },
}

#[derive(Resource)]
pub struct PlayerDying {
    phase: DeathPhase,
}

#[derive(Resource)]
struct DeathScaleLayer(ScaleLayerId);

#[derive(Component)]
struct ShrinkToZero {
    elapsed: f32,
    duration: f32,
}

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>()
            .add_systems(Startup, register_death_layer)
            .add_systems(OnEnter(WavePhase::Combat), init_run)
            .add_systems(
                Update,
                (
                    tick_run,
                    drain_life.run_if(not(resource_exists::<PlayerDying>)),
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

fn register_death_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(DeathScaleLayer(registry.register()));
}

fn init_run(mut run_state: ResMut<RunState>) {
    run_state.elapsed = 0.0;
    run_state.attempt += 1;
    info!("Starting run #{}", run_state.attempt);
}

fn tick_run(time: Res<Time>, mut run_state: ResMut<RunState>) {
    run_state.elapsed += time.delta_secs();
}

fn drain_life(
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
        (Entity, Has<ScaleModifiers>),
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
            for (entity, has_modifiers) in &combat_entities {
                let mut ec = commands.entity(entity);
                ec.insert(ShrinkToZero {
                    elapsed: 0.0,
                    duration: 0.5,
                });
                if !has_modifiers {
                    ec.insert(ScaleModifiers::default());
                }
            }
        }
    }
}

fn mark_new_shrink_targets(
    mut commands: Commands,
    query: Query<
        (Entity, Has<ScaleModifiers>),
        (
            Or<(With<DespawnOnExit<WavePhase>>, With<SpawnSource>)>,
            Without<Player>,
            Without<ShrinkToZero>,
        ),
    >,
) {
    for (entity, has_modifiers) in &query {
        let mut ec = commands.entity(entity);
        ec.insert(ShrinkToZero {
            elapsed: 0.0,
            duration: 0.5,
        });
        if !has_modifiers {
            ec.insert(ScaleModifiers::default());
        }
    }
}

fn animate_shrink_to_zero(
    layer: Res<DeathScaleLayer>,
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShrinkToZero, &mut ScaleModifiers)>,
) {
    let dt = time.delta_secs();
    for (entity, mut shrink, mut modifiers) in &mut query {
        shrink.elapsed += dt;
        let t = (shrink.elapsed / shrink.duration).clamp(0.0, 1.0);
        modifiers.set(layer.0, Vec3::splat(1.0 - t));
        if t >= 1.0 {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

const PLAYER_DEATH_PARTICLE_LIFETIME: f32 = 0.5;

fn player_death_sequence(
    layer: Res<DeathScaleLayer>,
    mut commands: Commands,
    time: Res<Time>,
    mut dying: ResMut<PlayerDying>,
    mut player_query: Query<(Entity, &Transform, &Children, &mut ScaleModifiers), With<Player>>,
    anim_state_query: Query<&JumpWalkAnimationState>,
    children_query: Query<&Children>,
    shrink_query: Query<(), With<ShrinkToZero>>,
    mut next_phase: ResMut<NextState<WavePhase>>,
) {
    let dt = time.delta_secs();

    if let DeathPhase::Shrinking {
        ref mut elapsed,
        particle_lifetime,
        ..
    } = dying.phase
    {
        *elapsed += dt;
        if let Ok((_, _, _, mut modifiers)) = player_query.single_mut() {
            let t = (*elapsed / PLAYER_SHRINK_DURATION).clamp(0.0, 1.0);
            modifiers.set(layer.0, Vec3::splat(1.0 - t));
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

    let Ok((_player_entity, transform, player_children, _)) = player_query.single_mut() else {
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

    crate::particles::start_particles(&mut commands, "player_death", pos);

    let _ = player_children;
    let _ = transform;

    dying.phase = DeathPhase::Shrinking {
        elapsed: 0.0,
        particle_lifetime: PLAYER_DEATH_PARTICLE_LIFETIME,
    };
}

fn cleanup_player_dying(mut commands: Commands) {
    commands.remove_resource::<PlayerDying>();
}
