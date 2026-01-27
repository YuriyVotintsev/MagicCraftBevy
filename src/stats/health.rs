use bevy::prelude::*;

use super::{ComputedStats, StatRegistry};

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

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    #[allow(dead_code)]
    pub fn heal(&mut self, amount: f32, max: f32) {
        self.current = (self.current + amount).min(max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

pub fn sync_health_to_max_life(
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut Health, &ComputedStats), Changed<ComputedStats>>,
) {
    let Some(max_life_id) = stat_registry.get("max_life") else {
        return;
    };

    for (mut health, stats) in &mut query {
        let max_life = stats.get(max_life_id);
        if health.current > max_life {
            health.current = max_life;
        }
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


