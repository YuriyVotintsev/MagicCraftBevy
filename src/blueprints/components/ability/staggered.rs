use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::spawn::EntitySpawner;
use crate::blueprints::SpawnSource;
use crate::schedule::GameSet;
use crate::stats::ComputedStats;
use crate::GameState;

#[blueprint_component]
pub struct Staggered {
    pub interval: ScalarExpr,
    pub entities: Vec<EntityDef>,
}

#[derive(Component)]
pub struct StaggeredTimer {
    pub delay: f32,
    pub elapsed: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_staggered, update_staggered)
            .chain()
            .in_set(GameSet::BlueprintExecution)
            .run_if(in_state(GameState::Playing)),
    );
}

fn init_staggered(
    mut commands: Commands,
    query: Query<(Entity, &Staggered, &SpawnSource), Without<StaggeredTimer>>,
) {
    for (entity, staggered, source) in &query {
        let delay = source.index as f32 * staggered.interval;
        commands
            .entity(entity)
            .insert(StaggeredTimer { delay, elapsed: 0.0 });
    }
}

fn update_staggered(
    mut spawner: EntitySpawner,
    time: Res<Time>,
    mut query: Query<(Entity, &Staggered, &mut StaggeredTimer, &SpawnSource)>,
    stats_query: Query<&ComputedStats>,
    transforms: Query<&Transform, Without<Staggered>>,
) {
    for (entity, staggered, mut timer, source) in &mut query {
        timer.elapsed += time.delta_secs();
        if timer.elapsed >= timer.delay {
            spawner.spawn_triggered(
                entity,
                source,
                source.source,
                source.target,
                &staggered.entities,
                &stats_query,
                &transforms,
            );
            spawner.commands.entity(entity).despawn();
        }
    }
}
