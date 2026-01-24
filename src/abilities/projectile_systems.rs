use bevy::prelude::*;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::enemy::Enemy;
use super::effects::{Projectile, ProjectileVelocity};
use super::registry::EffectRegistry;
use super::context::ContextValue;

const PROJECTILE_SIZE: f32 = 15.0;
const ENEMY_SIZE: f32 = 30.0;

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &ProjectileVelocity), With<Projectile>>,
) {
    let half_width = ARENA_WIDTH / 2.0;
    let half_height = ARENA_HEIGHT / 2.0;

    for (entity, mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();

        if transform.translation.x.abs() > half_width
            || transform.translation.y.abs() > half_height
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn projectile_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    effect_registry: Res<EffectRegistry>,
) {
    for (projectile_entity, projectile_transform, projectile) in &projectile_query {
        for (enemy_entity, enemy_transform) in &enemy_query {
            let distance = projectile_transform
                .translation
                .truncate()
                .distance(enemy_transform.translation.truncate());

            if distance < (PROJECTILE_SIZE + ENEMY_SIZE) / 2.0 {
                let mut ctx = projectile.context.clone();
                ctx.set_param("target", ContextValue::Entity(enemy_entity));
                ctx.set_param("hit_position", ContextValue::Vec3(projectile_transform.translation));

                for effect_def in &projectile.on_hit_effects {
                    effect_registry.execute(effect_def, &ctx, &mut commands);
                }

                commands.entity(projectile_entity).despawn();
                commands.entity(enemy_entity).despawn();
                break;
            }
        }
    }
}
