use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::coords::COLLIDER_HALF_HEIGHT;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[blueprint_component]
pub struct DestroyEnemyProjectiles {
    pub size: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        destroy_enemy_projectiles_system
            .in_set(GameSet::BlueprintExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn destroy_enemy_projectiles_system(
    mut commands: Commands,
    query: Query<(&DestroyEnemyProjectiles, &SpawnSource, &Transform)>,
    spatial_query: SpatialQuery,
) {
    for (destroyer, source, transform) in &query {
        let position = transform.translation;

        let projectile_layer = match source.caster_faction {
            Faction::Player => GameLayer::EnemyProjectile,
            Faction::Enemy => GameLayer::PlayerProjectile,
        };

        let filter = SpatialQueryFilter::from_mask(projectile_layer);
        let shape = Collider::cylinder(destroyer.size / 2.0, COLLIDER_HALF_HEIGHT);
        let hits = spatial_query.shape_intersections(&shape, position, Quat::IDENTITY, &filter);

        for proj_entity in hits {
            if let Ok(mut entity_commands) = commands.get_entity(proj_entity) {
                entity_commands.despawn();
            }
        }
    }
}
