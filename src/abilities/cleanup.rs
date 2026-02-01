use bevy::prelude::*;
use super::activators::AbilityInstance;

pub fn cleanup_orphaned_abilities(
    mut commands: Commands,
    ability_query: Query<(Entity, &AbilityInstance)>,
    owner_query: Query<Entity, Without<AbilityInstance>>,
) {
    for (entity, instance) in &ability_query {
        if !owner_query.contains(instance.owner) {
            commands.entity(entity).despawn();
        }
    }
}
