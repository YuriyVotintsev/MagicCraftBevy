use bevy::prelude::*;
use crate::stats::Dead;

#[derive(Component, Clone, Copy)]
pub struct AttachedTo {
    pub owner: Entity,
}

fn on_owner_death(
    on: On<Add, Dead>,
    attached_query: Query<(Entity, &AttachedTo)>,
    mut commands: Commands,
) {
    let owner = on.event_target();
    for (entity, attached) in &attached_query {
        if attached.owner == owner {
            commands.entity(entity).despawn();
        }
    }
}

pub fn register_lifecycle_systems(app: &mut App) {
    app.add_observer(on_owner_death);
}
