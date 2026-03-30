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
    pub landed: bool,
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
            state.landed = false;
        } else if state.amplitude > 0.0 {
            if state.landed {
                state.amplitude = (state.amplitude - dt * 8.0).max(0.0);
                if state.amplitude == 0.0 {
                    state.phase = 0.0;
                    state.landed = false;
                }
            } else {
                let prev = (state.phase / std::f32::consts::PI).floor();
                state.phase += dt * anim.bounce_speed;
                let curr = (state.phase / std::f32::consts::PI).floor();
                if curr > prev {
                    state.phase = curr * std::f32::consts::PI;
                    state.landed = true;
                }
            }
        }

        let h = state.phase.sin().abs();
        let y = h * anim.bounce_height * state.amplitude;
        let tilt = 0.0;

        let ng = 1.0 - h;
        let squash = ng.powi(3) * 0.3;
        let stretch = h * ng.powi(2) * 1.2;
        let ss = (stretch - squash) * state.amplitude;
        let scale_y = 1.0 + ss;
        let scale_xz = 1.0 / scale_y.sqrt();

        transform.translation.y = y - 0.5 * (1.0 - scale_y);
        transform.rotation = Quat::from_rotation_z(tilt);
        transform.scale = Vec3::new(scale_xz, scale_y, scale_xz);
    }
}
