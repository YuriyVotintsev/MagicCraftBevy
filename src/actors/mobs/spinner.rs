use std::f32::consts::{FRAC_PI_2, TAU};

use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use super::super::components::{
    Caster, CircleSprite, Collider, DynamicBody, FindNearestEnemy, GameLayer, Health,
    OnDeathParticles, PendingDamage, SelfMoving, Shadow, Shape as ColliderShape, Size, Sprite,
    SpriteShape, Target,
};
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use crate::faction::Faction;
use crate::palette;
use crate::particles;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat, StatCalculators};

use super::spawn::{compute_stats, current_max_life, enemy_sprite_color};

const SPIKE_COUNT: usize = 6;
const SPIKE_OFFSET: f32 = 0.55;
const BODY_ELEVATION: f32 = 0.5;
const DECAY_RATE: f32 = 3.0;
const DAMAGE_INTERVAL: f32 = 0.25;
const TRAIL_THRESHOLD: f32 = 0.5;
const MAX_SPIN_SPEED: f32 = 10.0;
const SPIKE_GROWTH_MAX: f32 = 3.0;
const SQUISH_MIN: f32 = 0.5;
const CONTACT_RADIUS: f32 = 150.0;
const HIT_RADIUS: f32 = 150.0;

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

#[derive(Clone, Copy, PartialEq)]
enum SpinnerPhase {
    Idle,
    Windup,
    Charge,
    Cooldown,
}

#[derive(Component)]
pub struct Spinner {
    pub idle_duration: f32,
    pub windup_duration: f32,
    pub charge_duration: f32,
    pub cooldown_duration: f32,
    pub charge_speed: f32,
    pub spike_length: f32,

    phase: SpinnerPhase,
    elapsed: f32,
    spin_angle: f32,
    spin_speed: f32,
    spike_growth: f32,
    squish: f32,
    hit_player: bool,
    damage_cooldown: f32,
    spike_entities: [Entity; SPIKE_COUNT],
    trail_emitters: [Option<Entity>; SPIKE_COUNT],
    pre_charge_layers: Option<CollisionLayers>,
}

#[derive(Resource)]
pub struct SpinnerSquishScaleLayer(pub ScaleLayerId);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_scale_layer);
    app.add_systems(Update, spinner_tick.in_set(GameSet::MobAI));
    app.add_systems(PostUpdate, (init_spinner_visual, animate_spinner).chain());
    app.add_observer(on_remove_spinner);
}

fn register_scale_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(SpinnerSquishScaleLayer(registry.register()));
}

pub fn spawn_spinner(
    commands: &mut Commands,
    pos: Vec2,
    s: &SpinnerStats,
    calculators: &StatCalculators,
) -> Entity {
    let (modifiers, dirty, computed) = compute_stats(
        calculators,
        &[(Stat::MaxLifeFlat, s.hp), (Stat::PhysicalDamageFlat, s.damage)],
    );
    let hp = current_max_life(&computed);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn_empty().id();
    commands.entity(id).insert((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: s.size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        DynamicBody { mass: s.mass },
        Health { current: hp },
        FindNearestEnemy { size: 4000.0, center: id },
        Caster(id),
        OnDeathParticles { config: "enemy_death_large" },
        Spinner {
            idle_duration: s.idle_duration,
            windup_duration: s.windup_duration,
            charge_duration: s.charge_duration,
            cooldown_duration: s.cooldown_duration,
            charge_speed: s.charge_speed,
            spike_length: s.spike_length,
            phase: SpinnerPhase::Idle,
            elapsed: 0.0,
            spin_angle: 0.0,
            spin_speed: 0.0,
            spike_growth: 1.0,
            squish: 1.0,
            hit_player: false,
            damage_cooldown: 0.0,
            spike_entities: [Entity::PLACEHOLDER; SPIKE_COUNT],
            trail_emitters: [None; SPIKE_COUNT],
            pre_charge_layers: None,
        },
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

fn spinner_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Spinner,
            &Transform,
            Option<&Target>,
            &Faction,
            Option<&Size>,
            Option<&CollisionLayers>,
        ),
        Without<crate::wave::RiseFromGround>,
    >,
    stats_query: Query<&ComputedStats>,
    target_transforms: Query<&Transform>,
    spatial_query: SpatialQuery,
    mut pending: MessageWriter<PendingDamage>,
) {
    let dt = time.delta_secs();
    for (entity, mut spinner, transform, target, faction, size, current_layers) in &mut query {
        spinner.elapsed += dt;

        match spinner.phase {
            SpinnerPhase::Idle => {
                if spinner.elapsed >= spinner.idle_duration && target.is_some() {
                    spinner.phase = SpinnerPhase::Windup;
                    spinner.elapsed = 0.0;
                    spinner.damage_cooldown = 0.0;
                }
            }
            SpinnerPhase::Windup => {
                let t = (spinner.elapsed / spinner.windup_duration).clamp(0.0, 1.0);
                let ease = t * t;
                spinner.spin_speed = MAX_SPIN_SPEED * ease;
                spinner.spike_growth = 1.0 + (SPIKE_GROWTH_MAX - 1.0) * t;
                spinner.squish = 1.0 + (SQUISH_MIN - 1.0) * t;

                if t > TRAIL_THRESHOLD {
                    let pos = crate::coord::to_2d(transform.translation);
                    for i in 0..SPIKE_COUNT {
                        if spinner.trail_emitters[i].is_none() {
                            let emitter = particles::start_particles(&mut commands, "spinner_trail", pos);
                            spinner.trail_emitters[i] = Some(emitter);
                        }
                    }
                }

                if *faction == Faction::Enemy {
                    spinner.damage_cooldown -= dt;
                    if spinner.damage_cooldown <= 0.0 {
                        apply_area_damage(
                            entity,
                            transform,
                            size,
                            CONTACT_RADIUS,
                            &stats_query,
                            &spatial_query,
                            &mut pending,
                        );
                        spinner.damage_cooldown = DAMAGE_INTERVAL;
                    }
                }

                if spinner.elapsed >= spinner.windup_duration {
                    let target_entity = target.map(|t| t.0).unwrap_or(entity);
                    let target_pos = target_transforms
                        .get(target_entity)
                        .map(|t| crate::coord::to_2d(t.translation))
                        .unwrap_or_default();
                    let my_pos = crate::coord::to_2d(transform.translation);
                    let diff = target_pos - my_pos;
                    let direction = if diff.length_squared() > 1.0 {
                        diff.normalize()
                    } else {
                        Vec2::X
                    };

                    spinner.pre_charge_layers = current_layers.copied();
                    spinner.phase = SpinnerPhase::Charge;
                    spinner.elapsed = 0.0;
                    spinner.hit_player = false;

                    let charge_layers = CollisionLayers::new(
                        GameLayer::Enemy,
                        [GameLayer::Wall, GameLayer::PlayerProjectile],
                    );
                    commands.entity(entity).insert((
                        charge_layers,
                        SelfMoving,
                        LinearVelocity(crate::coord::ground_vel(direction * spinner.charge_speed)),
                    ));
                }
            }
            SpinnerPhase::Charge => {
                if *faction == Faction::Enemy && !spinner.hit_player {
                    let hits = apply_area_damage(
                        entity,
                        transform,
                        size,
                        HIT_RADIUS,
                        &stats_query,
                        &spatial_query,
                        &mut pending,
                    );
                    if hits > 0 {
                        spinner.hit_player = true;
                    }
                }

                if spinner.elapsed >= spinner.charge_duration {
                    spinner.phase = SpinnerPhase::Cooldown;
                    spinner.elapsed = 0.0;
                    let mut e = commands.entity(entity);
                    if let Some(layers) = spinner.pre_charge_layers.take() {
                        e.insert(layers);
                    }
                    e.insert(LinearVelocity(Vec3::ZERO));
                    e.remove::<SelfMoving>();
                }
            }
            SpinnerPhase::Cooldown => {
                if spinner.elapsed >= spinner.cooldown_duration {
                    spinner.phase = SpinnerPhase::Idle;
                    spinner.elapsed = 0.0;
                }
            }
        }
    }
}

fn apply_area_damage(
    entity: Entity,
    transform: &Transform,
    size: Option<&Size>,
    radius: f32,
    stats_query: &Query<&ComputedStats>,
    spatial_query: &SpatialQuery,
    pending: &mut MessageWriter<PendingDamage>,
) -> usize {
    let position = crate::coord::to_2d(transform.translation);
    let entity_radius = size.map_or(0.0, |s| s.value / 2.0);
    let damage = stats_query
        .get(entity)
        .map(|s| s.get(Stat::PhysicalDamageFlat))
        .unwrap_or(10.0);

    let filter = SpatialQueryFilter::from_mask(GameLayer::Player);
    let shape = avian3d::prelude::Collider::sphere(radius + entity_radius);
    let hits = spatial_query.shape_intersections(
        &shape,
        crate::coord::ground_pos(position),
        Quat::IDENTITY,
        &filter,
    );

    let count = hits.len();
    for target in hits {
        pending.write(PendingDamage { target, amount: damage, source: Some(entity) });
    }
    count
}

fn init_spinner_visual(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Spinner), Added<Spinner>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut spinner) in &mut query {
        let spike_color = palette::color("enemy_ability");
        let spike_material = materials.add(StandardMaterial {
            base_color: spike_color,
            unlit: true,
            ..default()
        });

        for i in 0..SPIKE_COUNT {
            let angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let spike_mesh = meshes.add(Cone::new(0.15, spinner.spike_length));
            let position = Vec3::new(
                angle.cos() * SPIKE_OFFSET,
                BODY_ELEVATION,
                -(angle.sin() * SPIKE_OFFSET),
            );
            let rotation = Quat::from_rotation_y(angle) * Quat::from_rotation_z(-FRAC_PI_2);

            let spike = commands
                .spawn((
                    Mesh3d(spike_mesh),
                    MeshMaterial3d(spike_material.clone()),
                    Transform::from_translation(position).with_rotation(rotation),
                ))
                .id();
            commands.entity(entity).add_child(spike);
            spinner.spike_entities[i] = spike;
        }
    }
}

fn animate_spinner(
    time: Res<Time>,
    squish_layer: Res<SpinnerSquishScaleLayer>,
    mut commands: Commands,
    mut state_query: Query<(&mut Spinner, &Transform, &Children, Option<&Size>)>,
    mut transform_query: Query<&mut Transform, Without<Spinner>>,
    mut scale_query: Query<&mut ScaleModifiers>,
    circle_query: Query<(), With<CircleSprite>>,
) {
    let dt = time.delta_secs();

    for (mut spinner, spinner_transform, children, size) in &mut state_query {
        let spinner_pos = crate::coord::to_2d(spinner_transform.translation);
        let entity_scale = size.map_or(1.0, |s| s.value / 2.0);

        let active = matches!(spinner.phase, SpinnerPhase::Windup | SpinnerPhase::Charge);
        if !active {
            spinner.spin_speed = lerp_toward(spinner.spin_speed, 0.0, DECAY_RATE * dt);
            spinner.spike_growth = lerp_toward(spinner.spike_growth, 1.0, DECAY_RATE * dt);
            spinner.squish = lerp_toward(spinner.squish, 1.0, DECAY_RATE * dt);

            for emitter in &mut spinner.trail_emitters {
                if let Some(e) = emitter.take() {
                    particles::stop_particles(&mut commands, e);
                }
            }
        }

        spinner.spin_angle += spinner.spin_speed * dt;

        let spike_entities = spinner.spike_entities;
        let trail_emitters = spinner.trail_emitters;
        let spike_length = spinner.spike_length;
        let spin_angle = spinner.spin_angle;
        let spike_growth = spinner.spike_growth;

        for (i, &spike_entity) in spike_entities.iter().enumerate() {
            let base_angle = i as f32 * TAU / SPIKE_COUNT as f32;
            let angle = base_angle + spin_angle;
            let local_offset = Vec2::new(angle.cos(), angle.sin()) * SPIKE_OFFSET;

            if let Ok(mut spike_transform) = transform_query.get_mut(spike_entity) {
                spike_transform.translation = Vec3::new(local_offset.x, BODY_ELEVATION, -local_offset.y);
                spike_transform.rotation =
                    Quat::from_rotation_y(angle) * Quat::from_rotation_z(-FRAC_PI_2);
                spike_transform.scale = Vec3::new(1.0, spike_growth, 1.0);
            }

            if let Some(emitter_entity) = trail_emitters[i] {
                let dir = Vec2::new(angle.cos(), angle.sin());
                let tip_local = dir * (SPIKE_OFFSET + spike_length / 2.0 * spike_growth);
                let world_pos = spinner_pos + tip_local * entity_scale;
                if let Ok(mut emitter_transform) = transform_query.get_mut(emitter_entity) {
                    emitter_transform.translation = crate::coord::ground_pos(world_pos);
                    emitter_transform.translation.y = BODY_ELEVATION * entity_scale;
                }
            }
        }

        let sq = spinner.squish;
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

fn on_remove_spinner(
    on: On<Remove, Spinner>,
    query: Query<&Spinner>,
    mut commands: Commands,
) {
    let entity = on.event_target();
    let Ok(spinner) = query.get(entity) else { return };
    for emitter in &spinner.trail_emitters {
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
