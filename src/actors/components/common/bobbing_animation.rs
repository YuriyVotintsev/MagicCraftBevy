use bevy::prelude::*;

#[derive(Component)]
pub struct BobbingAnimation {
    pub amplitude: f32,
    pub speed: f32,
    pub base_elevation: f32,
}

#[derive(Component, Default)]
pub struct BobbingState {
    pub phase: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, (init, animate));
}

fn init(mut commands: Commands, query: Query<Entity, Added<BobbingAnimation>>) {
    for entity in &query {
        commands.entity(entity).insert(BobbingState::default());
    }
}

fn animate(
    time: Res<Time>,
    mut query: Query<(&BobbingAnimation, &mut BobbingState, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (anim, mut state, mut transform) in &mut query {
        state.phase += dt * anim.speed * std::f32::consts::TAU;
        transform.translation.y = anim.base_elevation + anim.amplitude * state.phase.sin();
    }
}
