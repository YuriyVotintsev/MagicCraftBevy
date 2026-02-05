use bevy::prelude::*;
use super::activator_support::AbilityEntity;
use super::AbilitySource;

pub fn cleanup_orphaned_abilities(
    mut commands: Commands,
    ability_query: Query<(Entity, &AbilitySource), With<AbilityEntity>>,
    owner_query: Query<Entity, Without<AbilityEntity>>,
) {
    for (entity, source) in &ability_query {
        if !owner_query.contains(source.caster) {
            commands.entity(entity).despawn();
        }
    }
}
