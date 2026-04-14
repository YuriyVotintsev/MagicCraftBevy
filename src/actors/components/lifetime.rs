use bevy::prelude::*;

use crate::schedule::GameSet;
use crate::GameState;

#[derive(Component)]
pub struct Lifetime {
    pub remaining: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        tick_lifetime
            .in_set(GameSet::AbilityLifecycle)
            .run_if(in_state(GameState::Playing)),
    );
}

fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.remaining -= dt;
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
