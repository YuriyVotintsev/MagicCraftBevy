use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::abilities::context::TargetInfo;
use crate::abilities::spawn::EntitySpawner;
use crate::abilities::AbilitySource;
use crate::schedule::GameSet;
use crate::GameState;
use crate::stats::ComputedStats;

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
    mut spawner: EntitySpawner,
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
            spawner.spawn_triggered(
                entity,
                source,
                TargetInfo::from_position(falling.target_position),
                TargetInfo::EMPTY,
                &falling.entities,
                &stats_query,
                &transforms,
            );

            spawner.commands.entity(entity).despawn();
        }
    }
}
