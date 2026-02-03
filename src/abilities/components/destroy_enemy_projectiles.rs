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

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub size: ParamValueRaw,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub size: ParamValue,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> Def {
        Def {
            size: resolve_param_value(&self.size, stat_registry),
        }
    }
}

#[derive(Component)]
pub struct DestroyEnemyProjectiles {
    pub size: f32,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let size = def.size.evaluate_f32(ctx.stats);
    commands.insert(DestroyEnemyProjectiles { size });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        destroy_enemy_projectiles_system
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn destroy_enemy_projectiles_system(
    mut commands: Commands,
    query: Query<(&DestroyEnemyProjectiles, &AbilitySource, &Transform)>,
    spatial_query: SpatialQuery,
) {
    for (destroyer, source, transform) in &query {
        let position = transform.translation.truncate();

        let projectile_layer = match source.caster_faction {
            Faction::Player => GameLayer::EnemyProjectile,
            Faction::Enemy => GameLayer::PlayerProjectile,
        };

        let filter = SpatialQueryFilter::from_mask(projectile_layer);
        let shape = Collider::circle(destroyer.size / 2.0);
        let hits = spatial_query.shape_intersections(&shape, position, 0.0, &filter);

        for proj_entity in hits {
            if let Ok(mut entity_commands) = commands.get_entity(proj_entity) {
                entity_commands.despawn();
            }
        }
    }
}
