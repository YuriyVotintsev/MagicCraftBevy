use bevy::prelude::*;

use super::{CapsuleSprite, CircleSprite, Sprite};
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use super::super::combat::ShotFired;

#[derive(Component)]
pub struct ShootSquish {
    pub amplitude: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct ShootSquishState {
    pub timer: f32,
}

#[derive(Resource)]
pub struct ShootScaleLayer(pub ScaleLayerId);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_layer);
    app.add_systems(PostUpdate, (init_shoot_squish, animate).chain());
}

fn register_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(ShootScaleLayer(registry.register()));
}

fn init_shoot_squish(
    mut commands: Commands,
    query: Query<Entity, Added<ShootSquish>>,
) {
    for entity in &query {
        commands.entity(entity).insert(ShootSquishState { timer: 0.0 });
    }
}

pub fn animate(
    layer: Res<ShootScaleLayer>,
    time: Res<Time>,
    mut query: Query<(Entity, &ShootSquish, &mut ShootSquishState, &Children)>,
    shot_fired_query: Query<(), With<ShotFired>>,
    mut sprite_query: Query<
        (&mut Transform, &Sprite, &mut ScaleModifiers),
        Or<(With<CapsuleSprite>, With<CircleSprite>)>,
    >,
    capsule_query: Query<&CapsuleSprite>,
) {
    let dt = time.delta_secs();

    for (entity, squish, mut state, children) in &mut query {
        if shot_fired_query.contains(entity) {
            state.timer = squish.duration;
        }

        if state.timer > 0.0 {
            state.timer = (state.timer - dt).max(0.0);
        }

        let (scale_y, scale_xz) = if state.timer > 0.0 {
            let t = 1.0 - state.timer / squish.duration;
            let factor = 1.0 - squish.amplitude * (1.0 - t).powi(2);
            (factor, 1.0 / factor)
        } else {
            (1.0, 1.0)
        };

        for child in children.iter() {
            let Ok((mut transform, sprite, mut modifiers)) = sprite_query.get_mut(child) else {
                continue;
            };

            let mesh_half = capsule_query
                .get(child)
                .map(|c| c.half_length + 0.5)
                .unwrap_or(0.5);

            modifiers.set(layer.0, Vec3::new(scale_xz, scale_y, scale_xz));
            transform.translation.y = sprite.elevation - mesh_half * (1.0 - scale_y);
        }
    }
}
