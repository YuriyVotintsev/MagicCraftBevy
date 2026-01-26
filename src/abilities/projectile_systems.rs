use avian2d::prelude::*;
use bevy::prelude::*;

use crate::arena::Wall;
use crate::Faction;
use super::effects::Projectile;
use super::registry::EffectRegistry;
use super::context::ContextValue;

pub fn projectile_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    projectile_query: Query<(&Projectile, &Faction)>,
    target_query: Query<&Faction, Without<Projectile>>,
    wall_query: Query<(), With<Wall>>,
    effect_registry: Res<EffectRegistry>,
) {
    for event in collision_events.read() {
        let entity1 = event.collider1;
        let entity2 = event.collider2;

        let (projectile_entity, other_entity) =
            if projectile_query.contains(entity1) {
                (entity1, entity2)
            } else if projectile_query.contains(entity2) {
                (entity2, entity1)
            } else {
                continue;
            };

        if wall_query.contains(other_entity) {
            if let Ok(mut entity_commands) = commands.get_entity(projectile_entity) {
                entity_commands.despawn();
            }
            continue;
        }

        if projectile_query.contains(other_entity) {
            continue;
        }

        let Ok((projectile, proj_faction)) = projectile_query.get(projectile_entity) else {
            continue;
        };
        let Ok(target_faction) = target_query.get(other_entity) else {
            continue;
        };

        if proj_faction == target_faction {
            continue;
        }

        let mut ctx = projectile.context.clone();
        ctx.set_param("target", ContextValue::Entity(other_entity));

        for effect_def in &projectile.on_hit_effects {
            effect_registry.execute(effect_def, &ctx, &mut commands);
        }

        if let Ok(mut entity_commands) = commands.get_entity(projectile_entity) {
            entity_commands.despawn();
        }
    }
}
