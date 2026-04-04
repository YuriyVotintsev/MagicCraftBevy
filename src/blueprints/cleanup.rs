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
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
            continue;
        };
        if !owner_query.contains(caster_entity) {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}
