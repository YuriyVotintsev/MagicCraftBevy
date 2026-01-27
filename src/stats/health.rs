use bevy::prelude::*;


#[derive(Component)]
pub struct Dead;

#[derive(Message)]
pub struct DeathEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

pub fn death_system(
    mut commands: Commands,
    mut death_events: MessageWriter<DeathEvent>,
    query: Query<(Entity, &Health), (Changed<Health>, Without<Dead>)>,
) {
    for (entity, health) in &query {
        if health.is_dead() {
            death_events.write(DeathEvent { entity });
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(Dead);
            }
        }
    }
}

pub fn cleanup_dead(mut commands: Commands, query: Query<Entity, With<Dead>>) {
    for entity in &query {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
}


