use avian2d::prelude::*;
use bevy::prelude::*;

use crate::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::fsm::MobType;
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
    projectile_query: Query<&Projectile>,
    mob_query: Query<Entity, With<MobType>>,
    effect_registry: Res<EffectRegistry>,
) {
    for event in collision_events.read() {
        let entity1 = event.collider1;
        let entity2 = event.collider2;

        let (projectile_entity, mob_entity) =
            if projectile_query.contains(entity1) && mob_query.contains(entity2) {
                (entity1, entity2)
            } else if projectile_query.contains(entity2) && mob_query.contains(entity1) {
                (entity2, entity1)
            } else {
                continue;
            };

        let projectile = projectile_query.get(projectile_entity).unwrap();

        let mut ctx = projectile.context.clone();
        ctx.set_param("target", ContextValue::Entity(mob_entity));

        for effect_def in &projectile.on_hit_effects {
            effect_registry.execute(effect_def, &ctx, &mut commands);
        }

        commands.entity(projectile_entity).despawn();
    }
}
