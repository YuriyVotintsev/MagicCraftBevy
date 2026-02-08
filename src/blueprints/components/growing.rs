use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::schedule::GameSet;
use crate::GameState;

use super::lifetime::Lifetime;

#[blueprint_component]
pub struct Growing {
    pub start_size: ScalarExpr,
    pub end_size: ScalarExpr,
}

#[derive(Component, Default)]
pub struct GrowingProgress {
    pub elapsed: f32,
    pub duration: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        (init_growing_progress, tick_growing)
            .chain()
            .in_set(GameSet::AbilityExecution)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(PostUpdate, sync_growing_with_lifetime);
}

fn init_growing_progress(
    mut commands: Commands,
    query: Query<Entity, (With<Growing>, Without<GrowingProgress>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(GrowingProgress::default());
    }
}

fn tick_growing(time: Res<Time>, mut query: Query<(&Growing, &mut GrowingProgress, &mut Transform)>) {
    let dt = time.delta_secs();
    for (growing, mut progress, mut transform) in &mut query {
        if progress.duration <= 0.0 {
            transform.scale = Vec3::splat(growing.start_size / 2.0);
            continue;
        }
        progress.elapsed += dt;
        let t = (progress.elapsed / progress.duration).clamp(0.0, 1.0);
        let size = growing.start_size + (growing.end_size - growing.start_size) * t;
        transform.scale = Vec3::splat(size / 2.0);
    }
}

fn sync_growing_with_lifetime(
    mut query: Query<(&mut GrowingProgress, &Lifetime), Changed<Lifetime>>,
) {
    for (mut progress, lifetime) in &mut query {
        if progress.duration == 0.0 {
            progress.duration = lifetime.remaining;
        }
    }
}
