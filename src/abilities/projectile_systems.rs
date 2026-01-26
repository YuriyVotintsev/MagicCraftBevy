use avian2d::prelude::*;
use bevy::prelude::*;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::Faction;
use super::effects::Projectile;
use super::registry::EffectRegistry;
use super::context::ContextValue;

pub fn despawn_out_of_bounds_projectiles(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Projectile>>,
) {
    let half_width = ARENA_WIDTH / 2.0;
    let half_height = ARENA_HEIGHT / 2.0;

    for (entity, transform) in &query {
        if transform.translation.x.abs() > half_width
            || transform.translation.y.abs() > half_height
        {
            commands.entity(entity).despawn();
        }
    }
}

pub fn projectile_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    projectile_query: Query<(&Projectile, &Faction)>,
    target_query: Query<&Faction>,
    effect_registry: Res<EffectRegistry>,
) {
    for event in collision_events.read() {
        let entity1 = event.collider1;
        let entity2 = event.collider2;

        let (projectile_entity, target_entity) =
            if projectile_query.contains(entity1) && target_query.contains(entity2) {
                (entity1, entity2)
            } else if projectile_query.contains(entity2) && target_query.contains(entity1) {
                (entity2, entity1)
            } else {
                continue;
            };

        let Ok((projectile, proj_faction)) = projectile_query.get(projectile_entity) else {
            continue;
        };
        let Ok(target_faction) = target_query.get(target_entity) else {
            continue;
        };

        if proj_faction == target_faction {
            continue;
        }

        let mut ctx = projectile.context.clone();
        ctx.set_param("target", ContextValue::Entity(target_entity));

        for effect_def in &projectile.on_hit_effects {
            effect_registry.execute(effect_def, &ctx, &mut commands);
        }

        commands.entity(projectile_entity).despawn();
    }
}
