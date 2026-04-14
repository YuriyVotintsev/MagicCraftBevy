use avian3d::prelude::*;
use bevy::prelude::*;

use crate::arena::Wall;
use crate::faction::Faction;
use crate::particles;
use crate::schedule::GameSet;

#[derive(Component, Debug, Clone, Copy)]
pub struct OnCollisionParticles {
    pub config: &'static str,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        on_collision_particles_system
            .in_set(GameSet::AbilityExecution)
            .before(super::super::combat::projectile_collision_physics),
    );
}

fn on_collision_particles_system(
    mut commands: Commands,
    mut events: MessageReader<CollisionStart>,
    effect_query: Query<(&OnCollisionParticles, &Faction, &Transform)>,
    target_query: Query<&Faction>,
    wall_query: Query<(), With<Wall>>,
    mut processed: Local<bevy::platform::collections::HashSet<(Entity, Entity)>>,
) {
    processed.clear();
    for event in events.read() {
        let (proj_entity, other_entity) = if effect_query.contains(event.collider1) {
            (event.collider1, event.collider2)
        } else if effect_query.contains(event.collider2) {
            (event.collider2, event.collider1)
        } else { continue };

        if processed.contains(&(proj_entity, other_entity)) { continue }
        processed.insert((proj_entity, other_entity));

        if wall_query.contains(other_entity) { continue }
        if effect_query.contains(other_entity) { continue }

        let Ok((particles_eff, proj_faction, proj_transform)) = effect_query.get(proj_entity) else { continue };
        let Ok(target_faction) = target_query.get(other_entity) else { continue };
        if proj_faction == target_faction { continue }

        let pos = crate::coord::to_2d(proj_transform.translation);
        particles::start_particles(&mut commands, particles_eff.config, pos);
    }
}
