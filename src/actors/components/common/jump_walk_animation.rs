use bevy::prelude::*;

use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};

#[derive(Component)]
pub struct SelfMoving;

#[derive(Component)]
pub struct JumpWalkAnimation {
    pub bounce_height: f32,
    pub bounce_duration: f32,
    pub land_squish: f32,
    pub land_duration: f32,
}

#[derive(Component, Default)]
pub struct JumpWalkAnimationState {
    pub phase: f32,
    pub amplitude: f32,
    pub landed: bool,
}

#[derive(Resource)]
pub struct JumpScaleLayer(pub ScaleLayerId);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_layer);
    app.add_systems(PostUpdate, (init, animate));
}

fn register_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(JumpScaleLayer(registry.register()));
}

fn init(mut commands: Commands, query: Query<Entity, Added<JumpWalkAnimation>>) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(JumpWalkAnimationState::default());
    }
}

pub fn animate(
    layer: Res<JumpScaleLayer>,
    time: Res<Time>,
    mut query: Query<(
        &JumpWalkAnimation,
        &mut JumpWalkAnimationState,
        &mut Transform,
        &ChildOf,
        &mut ScaleModifiers,
    )>,
    moving_query: Query<(), With<SelfMoving>>,
) {
    let dt = time.delta_secs();
    for (anim, mut state, mut transform, child_of, mut modifiers) in &mut query {
        let moving = moving_query.get(child_of.parent()).is_ok();

        if moving {
            state.phase += dt * (std::f32::consts::PI / anim.bounce_duration);
            state.amplitude = (state.amplitude + dt * 8.0).min(1.0);
            state.landed = false;
        } else if state.amplitude > 0.0 {
            if state.landed {
                let decay = if anim.land_duration > 0.0 { 1.0 / anim.land_duration } else { 8.0 };
                state.amplitude = (state.amplitude - dt * decay).max(0.0);
                if state.amplitude == 0.0 {
                    state.phase = 0.0;
                    state.landed = false;
                }
            } else {
                let prev = (state.phase / std::f32::consts::PI).floor();
                state.phase += dt * (std::f32::consts::PI / anim.bounce_duration);
                let curr = (state.phase / std::f32::consts::PI).floor();
                if curr > prev {
                    state.phase = curr * std::f32::consts::PI;
                    state.landed = true;
                }
            }
        }

        let h = state.phase.sin().abs();
        let y = h * anim.bounce_height * state.amplitude;

        let ng = 1.0 - h;
        let squash = ng.powi(3) * anim.land_squish;
        let stretch = h * ng.powi(2) * anim.land_squish * 4.0;
        let ss = (stretch - squash) * state.amplitude;
        let scale_y = 1.0 + ss;
        let scale_xz = 1.0 / scale_y.sqrt();

        transform.translation.y = 0.5 + y - 0.5 * (1.0 - scale_y);
        modifiers.set(layer.0, Vec3::new(scale_xz, scale_y, scale_xz));
    }
}
