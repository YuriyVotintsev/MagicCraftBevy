use bevy::prelude::*;
use serde::Deserialize;

use crate::abilities::param::{ParamValue, ParamValueRaw, resolve_param_value};
use crate::abilities::spawn::SpawnContext;
use crate::abilities::Target;
use crate::abilities::AbilitySource;
use crate::abilities::entity_def::EntityDef;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::{ComputedStats, DEFAULT_STATS, StatRegistry};

#[derive(Debug, Clone, Deserialize)]
pub struct DefRaw {
    pub height: ParamValueRaw,
    pub duration: ParamValueRaw,
    #[serde(default)]
    pub entities: Vec<crate::abilities::entity_def::EntityDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub height: ParamValue,
    pub duration: ParamValue,
    pub entities: Vec<EntityDef>,
}

impl DefRaw {
    pub fn resolve(&self, stat_registry: &StatRegistry) -> Def {
        Def {
            height: resolve_param_value(&self.height, stat_registry),
            duration: resolve_param_value(&self.duration, stat_registry),
            entities: self.entities.iter().map(|e| e.resolve(stat_registry)).collect(),
        }
    }
}

#[derive(Component)]
pub struct FallingProjectile {
    pub target_position: Vec3,
    pub height: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub entities: Vec<EntityDef>,
}

pub fn spawn(commands: &mut EntityCommands, def: &Def, ctx: &SpawnContext) {
    let height = def.height.evaluate_f32(ctx.stats);
    let duration = def.duration.evaluate_f32(ctx.stats);

    let target_position = match ctx.target {
        Some(Target::Point(p)) => p,
        _ => match ctx.source {
            Target::Point(p) => p,
            _ => Vec3::ZERO,
        },
    };

    commands.insert(FallingProjectile {
        target_position,
        height,
        duration,
        elapsed: 0.0,
        entities: def.entities.clone(),
    });
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        update_falling_projectiles
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn update_falling_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FallingProjectile, &AbilitySource, &mut Transform)>,
    stats_query: Query<&ComputedStats>,
) {
    let dt = time.delta_secs();

    for (entity, mut falling, source, mut transform) in &mut query {
        falling.elapsed += dt;
        let t = (falling.elapsed / falling.duration).clamp(0.0, 1.0);
        let eased_t = t * t;

        let start_y = falling.target_position.y + falling.height;
        let current_y = start_y - (falling.height * eased_t);
        transform.translation.x = falling.target_position.x;
        transform.translation.y = current_y;

        if t >= 1.0 {
            let caster_stats = stats_query
                .get(source.caster)
                .unwrap_or(&DEFAULT_STATS);

            let spawn_ctx = crate::abilities::spawn::SpawnContext {
                ability_id: source.ability_id,
                caster: source.caster,
                caster_faction: source.caster_faction,
                source: Target::Point(falling.target_position),
                target: None,
                stats: caster_stats,
                index: 0,
                count: 1,
            };

            for entity_def in &falling.entities {
                crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx);
            }

            commands.entity(entity).despawn();
        }
    }
}
