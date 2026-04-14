use bevy::prelude::*;

use crate::schedule::PostGameSet;
use crate::GameState;

use super::Health;

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct SkipCleanup;

#[derive(Message)]
pub struct DeathEvent {
    pub entity: Entity,
}

pub fn register_systems(app: &mut App) {
    app.add_message::<DeathEvent>()
        .add_systems(PostUpdate, death_system.in_set(PostGameSet))
        .add_systems(
            Last,
            cleanup_dead.run_if(not(in_state(GameState::Loading))),
        );
}

pub fn death_system(
    mut commands: Commands,
    mut death_events: MessageWriter<DeathEvent>,
    query: Query<(Entity, &Health), (Changed<Health>, Without<Dead>)>,
) {
    for (entity, health) in &query {
        if health.current <= 0.0 {
            death_events.write(DeathEvent { entity });
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(Dead);
            }
        }
    }
}

pub fn cleanup_dead(mut commands: Commands, query: Query<Entity, (With<Dead>, Without<SkipCleanup>)>) {
    for entity in &query {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
}
