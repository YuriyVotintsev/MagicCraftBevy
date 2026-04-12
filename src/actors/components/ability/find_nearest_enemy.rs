use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::SpawnSource;
use crate::actors::TargetInfo;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[derive(Component)]
pub struct FindNearestEnemy {
    pub size: f32,
    pub center: Entity,
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
    mut query: Query<(Entity, &FindNearestEnemy, &mut SpawnSource), Without<FoundTarget>>,
    spatial_query: SpatialQuery,
    transforms: Query<&Transform>,
) {
    for (entity, finder, mut source) in &mut query {
        let caster_pos = transforms
            .get(finder.center)
            .map(|t| crate::coord::to_2d(t.translation))
            .unwrap_or(Vec2::ZERO);

        let target_layer = match source.caster_faction {
            Faction::Player => GameLayer::Enemy,
            Faction::Enemy => GameLayer::Player,
        };

        let filter = SpatialQueryFilter::from_mask(target_layer);
        let shape = Collider::sphere(finder.size / 2.0);
        let hits = spatial_query.shape_intersections(&shape, crate::coord::ground_pos(caster_pos), Quat::IDENTITY, &filter);

        let target = hits
            .iter()
            .filter_map(|&e| {
                let pos = crate::coord::to_2d(transforms.get(e).ok()?.translation);
                Some((caster_pos.distance_squared(pos), e, pos))
            })
            .min_by(|(dist_a, _, _), (dist_b, _, _)| dist_a.total_cmp(dist_b))
            .map(|(_, e, pos)| (e, pos));

        if let Some((target_entity, target_pos)) = target {
            source.target = TargetInfo::from_entity_and_position(target_entity, target_pos);
            commands.entity(entity).insert(FoundTarget(target_entity, crate::coord::ground_pos(target_pos)));
        }
    }
}
