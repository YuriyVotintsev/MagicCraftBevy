use bevy::prelude::*;

use crate::wave::remove_invulnerability;
use crate::Faction;

use super::effects::{Projectile, ShieldActive, ShieldVisual};

pub fn update_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut shield_query: Query<(Entity, &mut ShieldActive, &Transform)>,
    projectile_query: Query<(Entity, &Transform, &Faction), With<Projectile>>,
) {
    for (entity, mut shield, shield_transform) in &mut shield_query {
        let shield_pos = shield_transform.translation.truncate();

        for (proj_entity, proj_transform, proj_faction) in &projectile_query {
            if *proj_faction != Faction::Enemy {
                continue;
            }

            let proj_pos = proj_transform.translation.truncate();
            let distance = shield_pos.distance(proj_pos);

            if distance <= shield.radius {
                if let Ok(mut entity_commands) = commands.get_entity(proj_entity) {
                    entity_commands.despawn();
                }
            }
        }

        if shield.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).remove::<ShieldActive>();
            remove_invulnerability(&mut commands, entity);
        }
    }
}

pub fn update_shield_visual(
    mut commands: Commands,
    shield_query: Query<&Transform, With<ShieldActive>>,
    mut visual_query: Query<(Entity, &ShieldVisual, &mut Transform), Without<ShieldActive>>,
) {
    for (visual_entity, visual, mut visual_transform) in &mut visual_query {
        if let Ok(owner_transform) = shield_query.get(visual.owner) {
            visual_transform.translation = owner_transform.translation.with_z(0.5);
        } else if let Ok(mut entity_commands) = commands.get_entity(visual_entity) {
            entity_commands.despawn();
        }
    }
}
