use bevy::prelude::*;
use crate::actors::combat::Dead;

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
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

pub fn register_lifecycle_systems(app: &mut App) {
    app.add_observer(on_owner_death);
}
