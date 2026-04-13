use std::f32::consts::{FRAC_PI_2, TAU};

use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::actors::combat::{Health, PendingDamage};
use crate::actors::components::ability::find_nearest_enemy::FindNearestEnemy;
use crate::actors::components::common::collider::{Collider, GameLayer, Shape as ColliderShape};
use crate::actors::components::common::dynamic_body::DynamicBody;
use crate::actors::components::common::jump_walk_animation::SelfMoving;
use crate::actors::components::common::shadow::Shadow;
use crate::actors::components::common::size::Size;
use crate::actors::components::common::sprite::{CircleSprite, Sprite, SpriteShape};
use crate::actors::effects::OnDeathParticles;
use crate::actors::SpawnSource;
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use crate::faction::Faction;
use crate::palette;
use crate::particles;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat, StatCalculators};

use super::{compute_stats, current_max_life, enemy_sprite_color};

const SPIKE_COUNT: usize = 6;
const SPIKE_OFFSET: f32 = 0.55;
const BODY_ELEVATION: f32 = 0.5;
const DECAY_RATE: f32 = 3.0;
const DAMAGE_INTERVAL: f32 = 0.25;
const TRAIL_THRESHOLD: f32 = 0.5;

#[derive(Clone, Deserialize, Debug)]
pub struct SpinnerStats {
    pub hp: f32,
    pub damage: f32,
    pub size: f32,
    pub mass: f32,
    pub spike_length: f32,
    pub idle_duration: f32,
    pub windup_duration: f32,
    pub charge_duration: f32,
    pub cooldown_duration: f32,
    pub charge_speed: f32,
}

#[derive(Component)]
pub struct SpinnerAi {
    pub idle_duration: f32,
    pub windup_duration: f32,
    pub charge_duration: f32,
    pub cooldown_duration: f32,
    pub charge_speed: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum SpinnerPhase {
    Idle,
    Windup,
    Charge,
    Cooldown,
}

#[derive(Component)]
pub struct SpinnerAiState {
    phase: SpinnerPhase,
    elapsed: f32,
}

#[derive(Component)]
pub struct SpinnerWindup {
    pub duration: f32,
    pub max_spin_speed: f32,
    pub spike_growth_max: f32,
    pub squish_min: f32,
    pub contact_radius: f32,
}

#[derive(Component)]
pub struct SpinnerWindupTimer {
    pub elapsed: f32,
    pub damage_cooldown: f32,
}

#[derive(Component)]
pub struct SpinnerCharge {
    pub speed: f32,
    pub max_duration: f32,
    pub hit_radius: f32,
    pub target: Entity,
}

#[derive(Component)]
pub struct SpinnerChargeState {
    pub elapsed: f32,
    pub hit_player: bool,
}

#[derive(Component)]
pub struct PreChargeLayers(pub CollisionLayers);

#[derive(Component)]
pub struct SpinnerVisual {
    pub spike_length: f32,
}

#[derive(Component)]
pub struct SpinnerVisualState {
    pub spike_entities: [Entity; SPIKE_COUNT],
    pub spin_angle: f32,
    pub spin_speed: f32,
    pub spike_growth: f32,
    pub squish: f32,
    pub trail_emitters: [Option<Entity>; SPIKE_COUNT],
}

#[derive(Resource)]
pub struct SpinnerSquishScaleLayer(pub ScaleLayerId);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_scale_layer);
    app.add_systems(
        Update,
        (
            init_spinner_ai,
            spinner_ai_system,
            init_windup,
            windup_system,
            init_charge,
            charge_system,
        )
            .chain()
            .in_set(GameSet::MobAI),
    );
    app.add_systems(PostUpdate, (init_spinner_visual, animate_spinner).chain());
    app.add_observer(on_remove_windup);
    app.add_observer(on_remove_charge);
    app.add_observer(on_remove_visual_state);
}

fn register_scale_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(SpinnerSquishScaleLayer(registry.register()));
}

pub fn spawn_spinner(
    commands: &mut Commands,
    pos: Vec2,
    s: &SpinnerStats,
    calculators: &StatCalculators,
    extra_modifiers: &[(Stat, f32)],
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        calculators,
        &[(Stat::MaxLifeFlat, s.hp), (Stat::PhysicalDamageFlat, s.damage)],
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
        SpinnerVisual { spike_length: s.spike_length },
        FindNearestEnemy { size: 4000.0, center: Entity::PLACEHOLDER },
        SpinnerAi {
            idle_duration: s.idle_duration,
            windup_duration: s.windup_duration,
            charge_duration: s.charge_duration,
            cooldown_duration: s.cooldown_duration,
            charge_speed: s.charge_speed,
        },
    )).id();

    commands.entity(id).insert((
        SpawnSource::from_caster(id, pos),
        FindNearestEnemy { size: 4000.0, center: id },
        OnDeathParticles { config: "enemy_death_large" },
    ));

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow { opacity: 0.45 });
        p.spawn(Sprite {
            color: enemy_sprite_color(), shape: SpriteShape::Circle,
            position: Vec2::ZERO, scale: 1.0, elevation: 0.5, half_length: 0.5,
        });
    });
    id
}

fn init_spinner_ai(
    mut commands: Commands,
    query: Query<Entity, Added<SpinnerAi>>,
) {
    for entity in &query {
        commands.entity(entity).insert(SpinnerAiState {
            phase: SpinnerPhase::Idle,
            elapsed: 0.0,
        });
    }
}

fn spinner_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &SpinnerAi, &mut SpinnerAiState, &SpawnSource), Without<crate::wave::summoning::RiseFromGround>>,
) {
    let dt = time.delta_secs();
    for (entity, ai, mut state, source) in &mut query {
        state.elapsed += dt;
        match state.phase {
            SpinnerPhase::Idle => {
                if state.elapsed >= ai.idle_duration && source.target.entity.is_some() {
                    state.phase = SpinnerPhase::Windup;
                    state.elapsed = 0.0;
                    commands.entity(entity).insert(SpinnerWindup {
                        duration: ai.windup_duration,
                        max_spin_speed: 10.0,
                        spike_growth_max: 3.0,
                        squish_min: 0.5,
                        contact_radius: 150.0,
                    });
                }
            }
            SpinnerPhase::Windup => {
                if state.elapsed >= ai.windup_duration {
                    state.phase = SpinnerPhase::Charge;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<SpinnerWindup>();
                    let target_entity = source.target.entity.unwrap_or(entity);
                    commands.entity(entity).insert(SpinnerCharge {
                        speed: ai.charge_speed,
                        max_duration: ai.charge_duration,
                        hit_radius: 150.0,
                        target: target_entity,
                    });
                }
            }
            SpinnerPhase::Charge => {
                if state.elapsed >= ai.charge_duration {
                    state.phase = SpinnerPhase::Cooldown;
                    state.elapsed = 0.0;
                    commands.entity(entity).remove::<SpinnerCharge>();
                }
            }
            SpinnerPhase::Cooldown => {
                if state.elapsed >= ai.cooldown_duration {
                    state.phase = SpinnerPhase::Idle;
                    state.elapsed = 0.0;
                }
            }
        }
    }
}

fn init_windup(mut commands: Commands, query: Query<Entity, Added<SpinnerWindup>>) {
    for entity in &query {
        commands.entity(entity).insert(SpinnerWindupTimer {
            elapsed: 0.0,
            damage_cooldown: 0.0,
        });
    }
}

fn windup_system(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &SpinnerWindup,
        &mut SpinnerWindupTimer,
        &mut SpinnerVisualState,
        &Transform,
        &Faction,
        Option<&Size>,
    )>,
    stats_query: Query<&ComputedStats>,
    spatial_query: SpatialQuery,
    mut pending: MessageWriter<PendingDamage>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, windup, mut timer, mut visual_state, transform, faction, size) in &mut query {
        timer.elapsed += dt;
        let t = (timer.elapsed / windup.duration).clamp(0.0, 1.0);
        let ease = t * t;

        visual_state.spin_speed = windup.max_spin_speed * ease;
        visual_state.spike_growth = 1.0 + (windup.spike_growth_max - 1.0) * t;
        visual_state.squish = 1.0 + (windup.squish_min - 1.0) * t;

        if t > TRAIL_THRESHOLD {
            let pos = crate::coord::to_2d(transform.translation);
            for i in 0..visual_state.trail_emitters.len() {
                if visual_state.trail_emitters[i].is_none() {
                    let emitter_entity = particles::start_particles(&mut commands, "spinner_trail", pos);
                    visual_state.trail_emitters[i] = Some(emitter_entity);
                }
            }
        }

        if *faction != Faction::Enemy {
            continue;
        }
        timer.damage_cooldown -= dt;
        if timer.damage_cooldown > 0.0 {
            continue;
        }

        let position = crate::coord::to_2d(transform.translation);
        let entity_radius = size.map_or(0.0, |s| s.value / 2.0);

        let damage = stats_query
            .get(entity)
            .map(|s| s.get(Stat::PhysicalDamageFlat))
            .unwrap_or(windup.contact_radius);

        let filter = SpatialQueryFilter::from_mask(GameLayer::Player);
        let shape = avian3d::prelude::Collider::sphere(windup.contact_radius + entity_radius);
        let hits = spatial_query.shape_intersections(
            &shape,
            crate::coord::ground_pos(position),
            Quat::IDENTITY,
            &filter,
        );

        for target in hits {
            pending.write(PendingDamage {
                target,
                amount: damage,
                source: Some(entity),
            });
        }
        timer.damage_cooldown = DAMAGE_INTERVAL;
    }
}

fn on_remove_windup(
    on: On<Remove, SpinnerWindup>,
    mut commands: Commands,
) {
    let entity = on.event_target();
    commands
        .entity(entity)
        .queue_silenced(|mut e: EntityWorldMut| {
            e.remove::<SpinnerWindupTimer>();
        });
}

fn init_charge(
    mut commands: Commands,
    query: Query<(Entity, &SpinnerCharge, &Transform), Added<SpinnerCharge>>,
    target_transforms: Query<&Transform, Without<SpinnerCharge>>,
    collision_query: Query<&CollisionLayers>,
) {
    for (entity, charge, transform) in &query {
        let target_pos = target_transforms
            .get(charge.target)
            .map(|t| crate::coord::to_2d(t.translation))
            .unwrap_or_default();

        let my_pos = crate::coord::to_2d(transform.translation);
        let diff = target_pos - my_pos;
        let direction = if diff.length_squared() > 1.0 {
            diff.normalize()
        } else {
            Vec2::X
        };

        let current_layers = collision_query
            .get(entity)
            .copied()
            .unwrap_or_default();

        let charge_layers = CollisionLayers::new(
            GameLayer::Enemy,
            [GameLayer::Wall, GameLayer::PlayerProjectile],
        );

        commands.entity(entity).insert((
            SpinnerChargeState {
                elapsed: 0.0,
                hit_player: false,
            },
            PreChargeLayers(current_layers),
            charge_layers,
            SelfMoving,
            LinearVelocity(crate::coord::ground_vel(direction * charge.speed)),
        ));
    }
}

fn charge_system(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &SpinnerCharge,
        &mut SpinnerChargeState,
        &Transform,
        &LinearVelocity,
        &Faction,
        Option<&Size>,
    )>,
    stats_query: Query<&ComputedStats>,
    spatial_query: SpatialQuery,
    mut pending: MessageWriter<PendingDamage>,
) {
    let dt = time.delta_secs();

    for (entity, charge, mut state, transform, velocity, faction, size) in &mut query {
        state.elapsed += dt;

        if *faction == Faction::Enemy && !state.hit_player {
            let position = crate::coord::to_2d(transform.translation);
            let entity_radius = size.map_or(0.0, |s| s.value / 2.0);

            let damage = stats_query
                .get(entity)
                .map(|s| s.get(Stat::PhysicalDamageFlat))
                .unwrap_or(10.0);

            let filter = SpatialQueryFilter::from_mask(GameLayer::Player);
            let shape = avian3d::prelude::Collider::sphere(charge.hit_radius + entity_radius);
            let hits = spatial_query.shape_intersections(
                &shape,
                crate::coord::ground_pos(position),
                Quat::IDENTITY,
                &filter,
            );

            for target in hits {
                pending.write(PendingDamage {
                    target,
                    amount: damage,
                    source: Some(entity),
                });
                state.hit_player = true;
            }
        }

        let vel_2d = crate::coord::to_2d(velocity.0);
        let stopped = vel_2d.length_squared() < 1.0 && state.elapsed > 0.1;
        let timed_out = state.elapsed >= charge.max_duration;

        if stopped || timed_out {
            // AfterTime transition will handle the actual state change,
            // but we can also force it for wall-stop case
        }
    }
}

fn on_remove_charge(
    on: On<Remove, SpinnerCharge>,
    mut commands: Commands,
    layers_query: Query<&PreChargeLayers>,
    mut velocity_query: Query<&mut LinearVelocity>,
) {
    let entity = on.event_target();
    let restored = layers_query.get(entity).ok().map(|pre| pre.0);

    if let Ok(mut vel) = velocity_query.get_mut(entity) {
        vel.0 = Vec3::ZERO;
    }

    commands
        .entity(entity)
        .queue_silenced(move |mut e: EntityWorldMut| {
            if let Some(layers) = restored {
                e.insert(layers);
            }
            e.remove::<(SpinnerChargeState, PreChargeLayers, SelfMoving)>();
        });
}

fn init_spinner_visual(
    mut commands: Commands,
    query: Query<(Entity, &SpinnerVisual), Added<SpinnerVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, visual) in &query {
        let spike_color = palette::color("enemy_ability");
        let spike_material = materials.add(StandardMaterial {
            base_color: spike_color,
            unlit: true,
            ..default()
        });

        let mut spike_entities = [Entity::PLACEHOLDER; SPIKE_COUNT];
        for i in 0..SPIKE_COUNT {
            let angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let spike_mesh = meshes.add(Cone::new(0.15, visual.spike_length));
            let position = Vec3::new(
                angle.cos() * SPIKE_OFFSET,
                BODY_ELEVATION,
                -(angle.sin() * SPIKE_OFFSET),
            );
            let rotation =
                Quat::from_rotation_y(angle) * Quat::from_rotation_z(-FRAC_PI_2);

            let spike = commands
                .spawn((
                    Mesh3d(spike_mesh),
                    MeshMaterial3d(spike_material.clone()),
                    Transform::from_translation(position).with_rotation(rotation),
                ))
                .id();
            commands.entity(entity).add_child(spike);
            spike_entities[i] = spike;
        }

        commands.entity(entity).insert(
            SpinnerVisualState {
                spike_entities,
                spin_angle: 0.0,
                spin_speed: 0.0,
                spike_growth: 1.0,
                squish: 1.0,
                trail_emitters: [None; SPIKE_COUNT],
            },
        );
    }
}

fn animate_spinner(
    time: Res<Time>,
    squish_layer: Res<SpinnerSquishScaleLayer>,
    mut state_query: Query<(
        Entity,
        &mut SpinnerVisualState,
        &Transform,
        &SpinnerVisual,
        &Children,
        Option<&Size>,
    )>,
    mut transform_query: Query<&mut Transform, Without<SpinnerVisualState>>,
    mut scale_query: Query<&mut ScaleModifiers>,
    circle_query: Query<(), With<CircleSprite>>,
    has_windup: Query<(), With<SpinnerWindup>>,
    has_charge: Query<(), With<SpinnerCharge>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, mut state, spinner_transform, visual, children, size) in &mut state_query {
        let spinner_pos = crate::coord::to_2d(spinner_transform.translation);
        let entity_scale = size.map_or(1.0, |s| s.value / 2.0);

        let active = has_windup.contains(entity) || has_charge.contains(entity);
        if !active {
            state.spin_speed = lerp_toward(state.spin_speed, 0.0, DECAY_RATE * dt);
            state.spike_growth = lerp_toward(state.spike_growth, 1.0, DECAY_RATE * dt);
            state.squish = lerp_toward(state.squish, 1.0, DECAY_RATE * dt);

            for emitter in &mut state.trail_emitters {
                if let Some(e) = emitter.take() {
                    particles::stop_particles(&mut commands, e);
                }
            }
        }

        state.spin_angle += state.spin_speed * dt;

        for (i, &spike_entity) in state.spike_entities.iter().enumerate() {
            let base_angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let angle = base_angle + state.spin_angle;

            let local_offset = Vec2::new(angle.cos(), angle.sin()) * SPIKE_OFFSET;

            if let Ok(mut spike_transform) = transform_query.get_mut(spike_entity) {
                spike_transform.translation = Vec3::new(
                    local_offset.x,
                    BODY_ELEVATION,
                    -local_offset.y,
                );
                spike_transform.rotation =
                    Quat::from_rotation_y(angle) * Quat::from_rotation_z(-FRAC_PI_2);

                spike_transform.scale = Vec3::new(1.0, state.spike_growth, 1.0);
            }

            if let Some(emitter_entity) = state.trail_emitters[i] {
                let dir = Vec2::new(angle.cos(), angle.sin());
                let tip_local = dir * (SPIKE_OFFSET + visual.spike_length / 2.0 * state.spike_growth);
                let world_pos = spinner_pos + tip_local * entity_scale;
                if let Ok(mut emitter_transform) = transform_query.get_mut(emitter_entity) {
                    emitter_transform.translation = crate::coord::ground_pos(world_pos);
                    emitter_transform.translation.y = BODY_ELEVATION * entity_scale;
                }
            }
        }

        let sq = state.squish;
        let expand = 1.0 / sq.sqrt();
        let squish_scale = Vec3::new(expand, sq, expand);

        for child in children.iter() {
            if circle_query.contains(child) {
                if let Ok(mut body_modifiers) = scale_query.get_mut(child) {
                    body_modifiers.set(squish_layer.0, squish_scale);
                }
            }
        }
    }
}

fn on_remove_visual_state(
    on: On<Remove, SpinnerVisualState>,
    query: Query<&SpinnerVisualState>,
    mut commands: Commands,
) {
    let entity = on.event_target();
    let Ok(state) = query.get(entity) else { return };
    for emitter in &state.trail_emitters {
        if let Some(e) = *emitter {
            particles::stop_particles(&mut commands, e);
        }
    }
}

fn lerp_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    if (current - target).abs() <= max_delta {
        target
    } else if current > target {
        current - max_delta
    } else {
        current + max_delta
    }
}
