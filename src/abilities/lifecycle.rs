use bevy::prelude::*;
use crate::stats::Dead;
use crate::schedule::PostGameSet;
use crate::GameState;

#[derive(Component, Clone, Copy)]
pub struct AttachedTo {
    pub owner: Entity,
}

fn cleanup_attached_entities(
    mut commands: Commands,
    attached_query: Query<(Entity, &AttachedTo)>,
    owner_query: Query<(), Without<Dead>>,
) {
    for (entity, attached) in &attached_query {
        if owner_query.get(attached.owner).is_err() {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    }
}

pub fn register_lifecycle_systems(app: &mut App) {
    app.add_systems(
        PostUpdate,
        cleanup_attached_entities
            .in_set(PostGameSet)
            .after(crate::stats::death_system)
            .run_if(in_state(GameState::Playing)),
    );
}
