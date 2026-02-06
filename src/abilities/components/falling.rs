use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::spawn::SpawnContext;
use crate::abilities::AbilitySource;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::{ComputedStats, DEFAULT_STATS};

#[ability_component(SOURCE_POSITION)]
pub struct Falling {
    pub height: ScalarExpr,
    pub duration: ScalarExpr,
    #[default_expr("target.position")]
    pub target_position: VecExpr,
    #[default_expr("caster.position")]
    pub caster_position: VecExpr,
    pub entities: Vec<EntityDef>,
}

#[derive(Component, Default)]
pub struct FallingProgress {
    pub elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_falling_progress, update_falling_projectiles)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_falling_progress(
    mut commands: Commands,
    query: Query<Entity, (With<Falling>, Without<FallingProgress>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(FallingProgress::default());
    }
}

fn update_falling_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Falling, &mut FallingProgress, &AbilitySource, &mut Transform)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform, Without<Falling>>,
) {
    let dt = time.delta_secs();

    for (entity, falling, mut progress, source, mut transform) in &mut query {
        progress.elapsed += dt;
        let t = (progress.elapsed / falling.duration).clamp(0.0, 1.0);
        let eased_t = t * t;

        let start_y = falling.target_position.y + falling.height;
        let current_y = start_y - (falling.height * eased_t);
        transform.translation.x = falling.target_position.x;
        transform.translation.y = current_y;

        if t >= 1.0 {
            let caster_stats = stats_query
                .get(source.caster)
                .unwrap_or(&DEFAULT_STATS);

            let caster_pos = transforms.get(source.caster)
                .map(|t| t.translation.truncate())
                .unwrap_or(falling.caster_position);

            let spawn_ctx = SpawnContext {
                ability_id: source.ability_id,
                caster: source.caster,
                caster_position: caster_pos,
                caster_faction: source.caster_faction,
                source: TargetInfo::from_position(falling.target_position),
                target: TargetInfo::EMPTY,
                stats: caster_stats,
                index: 0,
                count: 1,
            };

            for entity_def in &falling.entities {
                crate::abilities::spawn::spawn_entity_def(&mut commands, entity_def, &spawn_ctx, None, None, None);
            }

            commands.entity(entity).despawn();
        }
    }
}
