use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::SpawnSource;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[blueprint_component]
pub struct FindNearestEnemy {
    pub size: ScalarExpr,
    #[default_expr("caster.entity")]
    pub center: EntityExpr,
}

#[derive(Component)]
pub struct FoundTarget(pub Entity, pub Vec3);

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        find_nearest_enemy_system
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn find_nearest_enemy_system(
    mut commands: Commands,
    query: Query<(Entity, &FindNearestEnemy, &SpawnSource), Without<FoundTarget>>,
    spatial_query: SpatialQuery,
    transforms: Query<&Transform>,
) {
    for (entity, finder, source) in &query {
        let caster_pos = transforms
            .get(finder.center)
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::circle(finder.size / 2.0);
        let hits = spatial_query.shape_intersections(&shape, caster_pos, 0.0, &filter);

        let target = hits
            .iter()
            .filter_map(|&e| {
                let pos = transforms.get(e).ok()?.translation.truncate();
                Some((caster_pos.distance_squared(pos), e, pos))
            })
            .min_by(|(dist_a, _, _), (dist_b, _, _)| dist_a.total_cmp(dist_b))
            .map(|(_, e, pos)| (e, pos.extend(0.0)));

        if let Some((target_entity, target_pos)) = target {
            commands.entity(entity).insert(FoundTarget(target_entity, target_pos));
        } else {
            commands.entity(entity).despawn();
        }
    }
}
