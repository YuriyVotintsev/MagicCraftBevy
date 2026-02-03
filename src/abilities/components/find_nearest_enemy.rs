use avian2d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;
use crate::stats::StatRegistry;

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub size: ParamValueRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub size: ParamValue,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> Def {
        Def {
            size: resolve_param_value(&self.size, stat_registry),
        }
    }
}

#[derive(Component)]
pub struct FindNearestEnemy {
    pub size: f32,
}

#[derive(Component)]
pub struct FoundTarget(pub Entity, pub Vec3);

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let size = def.size.evaluate_f32(ctx.stats);
    commands.insert(FindNearestEnemy { size });
}

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
    query: Query<(Entity, &FindNearestEnemy, &AbilitySource), Without<FoundTarget>>,
    spatial_query: SpatialQuery,
    transforms: Query<&Transform>,
) {
    for (entity, finder, source) in &query {
        let caster_pos = transforms
            .get(source.caster)
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
