use avian3d::prelude::*;
use bevy::prelude::*;

use super::Target;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[derive(Component)]
pub struct FindNearestEnemy {
    pub size: f32,
    pub center: Entity,
}

#[derive(Component)]
pub struct FoundTarget;

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
    query: Query<(Entity, &FindNearestEnemy, &Faction), Without<FoundTarget>>,
    spatial_query: SpatialQuery,
    transforms: Query<&Transform>,
) {
    for (entity, finder, faction) in &query {
        let caster_pos = transforms
            .get(finder.center)
            .map(|t| crate::coord::to_2d(t.translation))
            .unwrap_or(Vec2::ZERO);

        let filter = SpatialQueryFilter::from_mask(faction.enemy_layer());
        let shape = Collider::sphere(finder.size / 2.0);
        let hits = spatial_query.shape_intersections(&shape, crate::coord::ground_pos(caster_pos), Quat::IDENTITY, &filter);

        let nearest = hits
            .iter()
            .filter_map(|&e| {
                let pos = crate::coord::to_2d(transforms.get(e).ok()?.translation);
                Some((caster_pos.distance_squared(pos), e))
            })
            .min_by(|(dist_a, _), (dist_b, _)| dist_a.total_cmp(dist_b))
            .map(|(_, e)| e);

        if let Some(target_entity) = nearest {
            commands.entity(entity).insert((Target(target_entity), FoundTarget));
        }
    }
}
