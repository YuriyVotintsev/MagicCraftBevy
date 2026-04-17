use bevy::prelude::*;

use super::super::lifetime::Lifetime;

#[derive(Component)]
pub struct ScaleOut {}

#[derive(Component)]
pub struct ScaleOutState {
    pub total: f32,
    pub start_scale: Vec3,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, (init_scale_out, update_scale_out).chain());
}

fn init_scale_out(
    mut commands: Commands,
    query: Query<(Entity, &Lifetime, &Transform), (With<ScaleOut>, Without<ScaleOutState>)>,
) {
    for (entity, lifetime, transform) in &query {
        commands.entity(entity).insert(ScaleOutState {
            total: lifetime.remaining,
            start_scale: transform.scale,
        });
    }
}

fn update_scale_out(
    mut query: Query<(&Lifetime, &ScaleOutState, &mut Transform), With<ScaleOut>>,
) {
    for (lifetime, state, mut transform) in &mut query {
        let t = (lifetime.remaining / state.total).clamp(0.0, 1.0);
        transform.scale = state.start_scale * t;
    }
}
