use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

#[blueprint_component]
pub struct JumpWalkAnimation {
    #[raw(default = 0.15)]
    pub bounce_height: ScalarExpr,
    #[raw(default = 10.0)]
    pub bounce_speed: ScalarExpr,
    #[raw(default = 12.0)]
    pub max_tilt: ScalarExpr,
}

#[derive(Component, Default)]
pub struct JumpWalkAnimationState {
    pub phase: f32,
    pub amplitude: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, (init, animate));
}

fn init(mut commands: Commands, query: Query<Entity, Added<JumpWalkAnimation>>) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(JumpWalkAnimationState::default());
    }
}

pub fn animate(
    time: Res<Time>,
    mut query: Query<(
        &JumpWalkAnimation,
        &mut JumpWalkAnimationState,
        &mut Transform,
        &ChildOf,
    )>,
    velocity_query: Query<&LinearVelocity>,
) {
    let dt = time.delta_secs();
    for (anim, mut state, mut transform, child_of) in &mut query {
        let moving = velocity_query
            .get(child_of.parent())
            .map(|v| v.length_squared() > 1.0)
            .unwrap_or(false);

        if moving {
            state.phase += dt * anim.bounce_speed;
            state.amplitude = (state.amplitude + dt * 8.0).min(1.0);
        } else {
            state.amplitude = (state.amplitude - dt * 8.0).max(0.0);
            if state.amplitude == 0.0 {
                state.phase = 0.0;
            }
        }

        let y = state.phase.sin().abs() * anim.bounce_height * state.amplitude;
        let tilt = state.phase.cos() * anim.max_tilt.to_radians() * state.amplitude;
        transform.translation.y = y;
        transform.rotation = Quat::from_rotation_z(tilt);
        transform.scale = Vec3::ONE;
    }
}
