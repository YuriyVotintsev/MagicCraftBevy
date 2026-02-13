use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use super::find_nearest_enemy::FoundTarget;
use crate::blueprints::SpawnSource;
use crate::physics::GameLayer;
use crate::schedule::GameSet;
use crate::Faction;
use crate::GameState;

#[blueprint_component]
pub struct FindRandomEnemy {
    pub size: ScalarExpr,
    #[default_expr("caster.entity")]
    pub center: EntityExpr,
    #[raw(default = false)]
    pub random_fallback: bool,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        find_random_enemy_system
            .in_set(GameSet::BlueprintExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn find_random_enemy_system(
    mut commands: Commands,
    query: Query<(Entity, &FindRandomEnemy, &SpawnSource), Without<FoundTarget>>,
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

        if hits.is_empty() {
            if finder.random_fallback {
                let angle = rand::random_range(0.0..std::f32::consts::TAU);
                let dist = rand::random_range(0.0..finder.size / 2.0);
                let pos = caster_pos + Vec2::new(angle.cos(), angle.sin()) * dist;
                commands.entity(entity).insert(FoundTarget(entity, pos.extend(0.0)));
            }
            continue;
        }

        let target_entity = hits[rand::random_range(0..hits.len())];
        if let Ok(transform) = transforms.get(target_entity) {
            commands
                .entity(entity)
                .insert(FoundTarget(target_entity, transform.translation));
        }
    }
}
