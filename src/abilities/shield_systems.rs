use avian2d::prelude::*;
use bevy::prelude::*;

use crate::physics::GameLayer;
use crate::wave::remove_invulnerability;
use crate::Faction;

use super::effects::{ShieldActive, ShieldVisual};

pub fn update_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut shield_query: Query<(Entity, &mut ShieldActive, &Transform)>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut shield, shield_transform) in &mut shield_query {
        let shield_pos = shield_transform.translation.truncate();

        let projectile_layer = match shield.owner_faction {
            Faction::Player => GameLayer::EnemyProjectile,
            Faction::Enemy => GameLayer::PlayerProjectile,
        };

        let filter = SpatialQueryFilter::from_mask(projectile_layer);
        let shape = Collider::circle(shield.radius);
        let hits = spatial_query.shape_intersections(&shape, shield_pos, 0.0, &filter);

        for proj_entity in hits {
            if let Ok(mut entity_commands) = commands.get_entity(proj_entity) {
                entity_commands.despawn();
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
