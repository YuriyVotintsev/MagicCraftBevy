use avian2d::prelude::*;
use bevy::prelude::*;

use crate::abilities::registry::{EffectHandler, EffectRegistry};
use crate::abilities::effect_def::EffectDef;
use crate::abilities::context::{AbilityContext, ContextValue};
use crate::abilities::events::ExecuteEffectEvent;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::Faction;
use crate::Lifetime;
use crate::GameState;

const METEOR_START_HEIGHT: f32 = 400.0;
const METEOR_SIZE: f32 = 40.0;
const EXPLOSION_DURATION: f32 = 0.3;

#[derive(Component)]
pub struct MeteorRequest {
    pub search_radius: f32,
    pub damage_radius: f32,
    pub fall_duration: f32,
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

#[derive(Component)]
pub struct MeteorFalling {
    pub target_position: Vec3,
    pub damage_radius: f32,
    pub fall_duration: f32,
    pub elapsed: f32,
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
}

#[derive(Component)]
pub struct MeteorIndicator;

#[derive(Component)]
pub struct MeteorExplosion {
    pub damage_radius: f32,
    pub on_hit_effects: Vec<EffectDef>,
    pub context: AbilityContext,
    pub damaged: bool,
}

fn execute_meteor_effect(
    mut commands: Commands,
    mut effect_events: MessageReader<ExecuteEffectEvent>,
    effect_registry: Res<EffectRegistry>,
    stats_query: Query<&ComputedStats>,
) {
    for event in effect_events.read() {
        let Some(handler_id) = effect_registry.get_id("spawn_meteor") else {
            continue;
        };
        if event.effect.effect_type != handler_id {
            continue;
        }

        let caster_stats = stats_query
            .get(event.context.caster)
            .ok()
            .cloned()
            .unwrap_or_default();

        let search_radius = event
            .effect
            .get_f32("search_radius", &caster_stats, &effect_registry)
            .unwrap_or(500.0);
        let damage_radius = event
            .effect
            .get_f32("damage_radius", &caster_stats, &effect_registry)
            .unwrap_or(80.0);
        let fall_duration = event
            .effect
            .get_f32("fall_duration", &caster_stats, &effect_registry)
            .unwrap_or(0.5);
        let on_hit_effects = event
            .effect
            .get_effect_list("on_hit", &effect_registry)
            .cloned()
            .unwrap_or_default();

        commands.spawn((
            Name::new("MeteorRequest"),
            MeteorRequest {
                search_radius,
                damage_radius,
                fall_duration,
                on_hit_effects,
                context: event.context.clone(),
            },
        ));
    }
}

#[derive(Default)]
pub struct SpawnMeteorHandler;

impl EffectHandler for SpawnMeteorHandler {
    fn name(&self) -> &'static str {
        "spawn_meteor"
    }

    fn register_execution_system(&self, app: &mut App) {
        app.add_systems(
            Update,
            execute_meteor_effect
                .in_set(GameSet::AbilityExecution)
                .run_if(in_state(GameState::Playing)),
        );
    }

    fn register_behavior_systems(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                meteor_target_finder,
                meteor_falling_update,
                meteor_explosion_damage,
            )
                .in_set(GameSet::AbilityExecution),
        );
    }
}

fn meteor_target_finder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &MeteorRequest)>,
    spatial_query: SpatialQuery,
    transforms: Query<&Transform>,
) {
    for (request_entity, request) in &query {
        let caster_pos = request.context.caster_position.truncate();

        let target_layer = match request.context.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(request.search_radius);
        let hits = spatial_query.shape_intersections(&shape, caster_pos, 0.0, &filter);

        let mut closest_enemy: Option<(Entity, f32, Vec2)> = None;

        for entity in hits {
            let Ok(transform) = transforms.get(entity) else {
                continue;
            };

            let enemy_pos = transform.translation.truncate();
            let dist_sq = caster_pos.distance_squared(enemy_pos);

            if let Some((_, current_dist_sq, _)) = closest_enemy {
                if dist_sq < current_dist_sq {
                    closest_enemy = Some((entity, dist_sq, enemy_pos));
                }
            } else {
                closest_enemy = Some((entity, dist_sq, enemy_pos));
            }
        }

        commands.entity(request_entity).despawn();

        let Some((_target_entity, _, target_pos)) = closest_enemy else {
            continue;
        };

        let target_position = Vec3::new(target_pos.x, target_pos.y, 0.0);

        let indicator_mesh = meshes.add(Circle::new(request.damage_radius));
        let indicator_material = materials.add(ColorMaterial::from_color(Color::srgba(1.0, 0.3, 0.0, 0.3)));

        commands.spawn((
            Name::new("MeteorIndicator"),
            MeteorIndicator,
            Mesh2d(indicator_mesh),
            MeshMaterial2d(indicator_material),
            Transform::from_translation(target_position.with_z(-1.0)),
            Lifetime { remaining: request.fall_duration + EXPLOSION_DURATION },
        ));

        let meteor_mesh = meshes.add(Circle::new(METEOR_SIZE / 2.0));
        let meteor_material = materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.5, 0.0)));

        let start_position = target_position + Vec3::new(0.0, METEOR_START_HEIGHT, 0.0);
        commands.spawn((
            Name::new("MeteorFalling"),
            MeteorFalling {
                target_position,
                damage_radius: request.damage_radius,
                fall_duration: request.fall_duration,
                elapsed: 0.0,
                on_hit_effects: request.on_hit_effects.clone(),
                context: request.context.clone(),
            },
            Mesh2d(meteor_mesh),
            MeshMaterial2d(meteor_material),
            Transform::from_translation(start_position),
        ));
    }
}

fn meteor_falling_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MeteorFalling, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut meteor, mut transform) in &mut query {
        meteor.elapsed += dt;
        let t = (meteor.elapsed / meteor.fall_duration).clamp(0.0, 1.0);
        let eased_t = t * t;

        let start_y = meteor.target_position.y + METEOR_START_HEIGHT;
        let current_y = start_y - (METEOR_START_HEIGHT * eased_t);
        transform.translation.y = current_y;

        if t >= 1.0 {
            commands.entity(entity).despawn();

            let explosion_mesh = meshes.add(Circle::new(meteor.damage_radius));
            let explosion_material = materials.add(ColorMaterial::from_color(Color::srgba(1.0, 0.5, 0.0, 0.8)));

            commands.spawn((
                Name::new("MeteorExplosion"),
                MeteorExplosion {
                    damage_radius: meteor.damage_radius,
                    on_hit_effects: meteor.on_hit_effects.clone(),
                    context: meteor.context.clone(),
                    damaged: false,
                },
                Mesh2d(explosion_mesh),
                MeshMaterial2d(explosion_material),
                Transform::from_translation(meteor.target_position),
                Lifetime { remaining: EXPLOSION_DURATION },
            ));
        }
    }
}

fn meteor_explosion_damage(
    mut query: Query<(&mut MeteorExplosion, &Transform)>,
    mut effect_events: MessageWriter<ExecuteEffectEvent>,
    spatial_query: SpatialQuery,
) {
    for (mut explosion, explosion_transform) in &mut query {
        if explosion.damaged {
            continue;
        }
        explosion.damaged = true;

        let explosion_pos = explosion_transform.translation.truncate();

        let target_layer = match explosion.context.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(explosion.damage_radius);
        let hits = spatial_query.shape_intersections(&shape, explosion_pos, 0.0, &filter);

        for enemy_entity in hits {
            let mut ctx = explosion.context.clone();
            ctx.set_param("target", ContextValue::Entity(enemy_entity));

            for effect_def in &explosion.on_hit_effects {
                effect_events.write(ExecuteEffectEvent {
                    effect: effect_def.clone(),
                    context: ctx.clone(),
                });
            }
        }
    }
}

register_effect!(SpawnMeteorHandler);
