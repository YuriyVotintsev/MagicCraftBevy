use bevy::prelude::*;
use super::activator_support::AbilityEntity;
use super::AbilitySource;

pub fn cleanup_orphaned_abilities(
    mut commands: Commands,
    ability_query: Query<(Entity, &AbilitySource), With<AbilityEntity>>,
    owner_query: Query<Entity, Without<AbilityEntity>>,
) {
    for (entity, source) in &ability_query {
        let Some(caster_entity) = source.caster.entity else {
            commands.entity(entity).despawn();
            continue;
        };
        if !owner_query.contains(caster_entity) {
            commands.entity(entity).despawn();
        }
    }
}
