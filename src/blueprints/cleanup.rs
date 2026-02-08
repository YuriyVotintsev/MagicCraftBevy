use bevy::prelude::*;
use super::core_components::BlueprintEntity;
use super::SpawnSource;

pub fn cleanup_orphaned_blueprint_entities(
    mut commands: Commands,
    blueprint_query: Query<(Entity, &SpawnSource), With<BlueprintEntity>>,
    owner_query: Query<Entity, Without<BlueprintEntity>>,
) {
    for (entity, source) in &blueprint_query {
        let Some(caster_entity) = source.caster.entity else {
            commands.entity(entity).despawn();
            continue;
        };
        if !owner_query.contains(caster_entity) {
            commands.entity(entity).despawn();
        }
    }
}
