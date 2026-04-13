use bevy::prelude::*;

use crate::actors::components::common::health::Health;
use crate::hit_flash::HitFlash;
use crate::schedule::{GameSet, PostGameSet};
use crate::stats::{ComputedStats, Stat};
use crate::wave::InvulnerableStack;
use crate::GameState;

#[derive(Message)]
pub struct PendingDamage {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct SkipCleanup;

#[derive(Message)]
pub struct DeathEvent {
    pub entity: Entity,
}

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

pub fn apply_pending_damage(
    mut commands: Commands,
    mut pending: MessageReader<PendingDamage>,
    mut query: Query<(&mut Health, &ComputedStats)>,
    stats_query: Query<&ComputedStats>,
    invulnerable: Query<(), With<InvulnerableStack>>,
) {
    for hit in pending.read() {
        if invulnerable.get(hit.target).is_ok() {
            continue;
        }

        let Ok((mut health, stats)) = query.get_mut(hit.target) else {
            continue;
        };

        let mut amount = hit.amount;

        if let Some(source_entity) = hit.source {
            if let Ok(source_stats) = stats_query.get(source_entity) {
                let chance = source_stats.get(Stat::CritChance);
                if rand::random::<f32>() < chance {
                    let multiplier = source_stats.get(Stat::CritMultiplier);
                    let effective = if multiplier > 0.0 { multiplier } else { 1.5 };
                    amount *= effective;
                }
            }
        }

        let max = stats.get(Stat::MaxLife).max(1.0);
        health.current = (health.current - amount).clamp(0.0, max);

        if let Ok(mut entity_commands) = commands.get_entity(hit.target) {
            entity_commands.insert(HitFlash::new());
        }
    }
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

pub fn register_systems(app: &mut App) {
    app.add_message::<PendingDamage>()
        .add_message::<DeathEvent>()
        .add_observer(on_owner_death)
        .add_systems(Update, apply_pending_damage.in_set(GameSet::DamageApply))
        .add_systems(PostUpdate, death_system.in_set(PostGameSet))
        .add_systems(
            Last,
            cleanup_dead.run_if(not(in_state(GameState::Loading))),
        );
}
