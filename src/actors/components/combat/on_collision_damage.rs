use avian3d::prelude::*;
use bevy::prelude::*;

use super::PendingDamage;
use super::Caster;
use crate::arena::Wall;
use crate::faction::Faction;
use crate::schedule::GameSet;

#[derive(Component, Debug, Clone, Copy)]
pub struct OnCollisionDamage {
    pub amount: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        on_collision_damage_system
            .in_set(GameSet::AbilityExecution)
            .before(super::projectile_collision_physics),
    );
}

fn on_collision_damage_system(
    mut events: MessageReader<CollisionStart>,
    mut pending: MessageWriter<PendingDamage>,
    damage_query: Query<(&OnCollisionDamage, &Faction, &Caster)>,
    target_query: Query<&Faction>,
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

        let Ok((dmg, proj_faction, caster)) = damage_query.get(proj_entity) else { continue };
        let Ok(target_faction) = target_query.get(other_entity) else { continue };
        if proj_faction == target_faction { continue }

        pending.write(PendingDamage {
            target: other_entity,
            amount: dmg.amount,
            source: Some(caster.0),
        });
    }
}
