use bevy::prelude::*;

use crate::Faction;
use crate::Lifetime;
use super::context::ContextValue;
use super::effects::{MeteorRequest, MeteorFalling, MeteorIndicator, MeteorExplosion, Projectile, OrbitingMovement};
use super::registry::EffectRegistry;

const METEOR_START_HEIGHT: f32 = 400.0;
const METEOR_SIZE: f32 = 40.0;
const EXPLOSION_DURATION: f32 = 0.3;

pub fn meteor_target_finder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &MeteorRequest)>,
    enemies: Query<(Entity, &Transform, &Faction), (Without<Projectile>, Without<OrbitingMovement>)>,
) {
    for (request_entity, request) in &query {
        let caster_pos = request.context.caster_position.truncate();
        let search_radius_sq = request.search_radius * request.search_radius;

        let opposite_faction = match request.context.caster_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => Faction::Player,
        };

        let mut closest_enemy: Option<(Entity, f32, Vec2)> = None;

        for (enemy_entity, enemy_transform, faction) in &enemies {
            if *faction != opposite_faction {
                continue;
            }

            let enemy_pos = enemy_transform.translation.truncate();
            let dist_sq = caster_pos.distance_squared(enemy_pos);

            if dist_sq > search_radius_sq {
                continue;
            }

            if let Some((_, current_dist_sq, _)) = closest_enemy {
                if dist_sq < current_dist_sq {
                    closest_enemy = Some((enemy_entity, dist_sq, enemy_pos));
                }
            } else {
                closest_enemy = Some((enemy_entity, dist_sq, enemy_pos));
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

pub fn meteor_falling_update(
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

pub fn meteor_explosion_damage(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MeteorExplosion, &Transform)>,
    enemies: Query<(Entity, &Transform, &Faction), (Without<Projectile>, Without<OrbitingMovement>)>,
    effect_registry: Res<EffectRegistry>,
) {
    for (_explosion_entity, mut explosion, explosion_transform) in &mut query {
        if explosion.damaged {
            continue;
        }
        explosion.damaged = true;

        let explosion_pos = explosion_transform.translation.truncate();
        let damage_radius_sq = explosion.damage_radius * explosion.damage_radius;

        let opposite_faction = match explosion.context.caster_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => Faction::Player,
        };

        for (enemy_entity, enemy_transform, faction) in &enemies {
            if *faction != opposite_faction {
                continue;
            }

            let enemy_pos = enemy_transform.translation.truncate();
            let dist_sq = explosion_pos.distance_squared(enemy_pos);

            if dist_sq > damage_radius_sq {
                continue;
            }

            let mut ctx = explosion.context.clone();
            ctx.set_param("target", ContextValue::Entity(enemy_entity));

            for effect_def in &explosion.on_hit_effects {
                effect_registry.execute(effect_def, &ctx, &mut commands);
            }
        }
    }
}
