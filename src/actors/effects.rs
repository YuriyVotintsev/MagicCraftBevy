use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::SpawnSource;
use crate::faction::Faction;
use crate::particles;
use crate::physics::Wall;
use crate::schedule::GameSet;
use crate::stats::{Dead, PendingDamage};

#[derive(Component, Debug, Clone)]
pub struct OnDeathParticles {
    pub config: &'static str,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct OnCollisionDamage {
    pub amount: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct OnCollisionParticles {
    pub config: &'static str,
}

pub fn register_systems(app: &mut App) {
    app.add_observer(on_death_particles_observer);
    app.add_systems(Update, on_collision_effects_system.in_set(GameSet::Damage));
}

fn on_death_particles_observer(
    on: On<Add, Dead>,
    mut commands: Commands,
    query: Query<(&OnDeathParticles, &Transform)>,
) {
    let entity = on.event_target();
    let Ok((effect, transform)) = query.get(entity) else { return };
    let pos = crate::coord::to_2d(transform.translation);
    particles::start_particles(&mut commands, effect.config, pos);
}

#[allow(clippy::too_many_arguments)]
fn on_collision_effects_system(
    mut commands: Commands,
    mut events: MessageReader<CollisionStart>,
    mut pending: MessageWriter<PendingDamage>,
    damage_query: Query<(Option<&OnCollisionDamage>, Option<&OnCollisionParticles>, &Faction, &Transform, &SpawnSource)>,
    target_query: Query<&Faction, Without<OnCollisionDamage>>,
    wall_query: Query<(), With<Wall>>,
    mut processed: Local<bevy::platform::collections::HashSet<(Entity, Entity)>>,
) {
    processed.clear();
    for event in events.read() {
        let (proj_entity, other_entity) = if damage_query.contains(event.collider1) {
            (event.collider1, event.collider2)
        } else if damage_query.contains(event.collider2) {
            (event.collider2, event.collider1)
        } else { continue };

        if processed.contains(&(proj_entity, other_entity)) { continue }
        processed.insert((proj_entity, other_entity));

        if wall_query.contains(other_entity) { continue }
        if damage_query.contains(other_entity) { continue }

        let Ok((dmg, particles_eff, proj_faction, proj_transform, source)) = damage_query.get(proj_entity) else { continue };
        let Ok(target_faction) = target_query.get(other_entity) else { continue };
        if proj_faction == target_faction { continue }

        if let Some(dmg) = dmg {
            pending.write(PendingDamage {
                target: other_entity,
                amount: dmg.amount,
                source: source.caster.entity,
            });
        }
        if let Some(particles_eff) = particles_eff {
            let pos = crate::coord::to_2d(proj_transform.translation);
            particles::start_particles(&mut commands, particles_eff.config, pos);
        }
    }
}
