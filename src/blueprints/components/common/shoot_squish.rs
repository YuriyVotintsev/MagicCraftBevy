use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use super::sprite::{CapsuleSprite, CircleSprite, Sprite, SquareSprite, TriangleSprite};
use crate::blueprints::components::common::jump_walk_animation::animate as jump_animate;
use crate::blueprints::components::common::squish_walk_animation::animate as squish_animate;
use crate::blueprints::components::mob::use_abilities::ShotFired;

#[blueprint_component]
pub struct ShootSquish {
    #[raw(default = 0.15)]
    pub amplitude: f32,
    #[raw(default = 0.2)]
    pub duration: f32,
}

#[derive(Component)]
pub struct ShootSquishState {
    pub timer: f32,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (init_shoot_squish, animate)
            .chain()
            .after(jump_animate)
            .after(squish_animate),
    );
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
    time: Res<Time>,
    mut query: Query<(Entity, &ShootSquish, &mut ShootSquishState, &Children)>,
    shot_fired_query: Query<(), With<ShotFired>>,
    mut sprite_query: Query<
        (&mut Transform, &Sprite),
        Or<(With<CapsuleSprite>, With<CircleSprite>, With<TriangleSprite>, With<SquareSprite>)>,
    >,
    capsule_query: Query<&CapsuleSprite>,
) {
    let dt = time.delta_secs();

    for (entity, squish, mut state, children) in &mut query {
        if shot_fired_query.contains(entity) {
            state.timer = squish.duration;
        }

        if state.timer <= 0.0 {
            continue;
        }

        state.timer = (state.timer - dt).max(0.0);

        let (scale_y, scale_xz) = if state.timer > 0.0 {
            let t = 1.0 - state.timer / squish.duration;
            let factor = 1.0 - squish.amplitude * (1.0 - t).powi(2);
            (factor, 1.0 / factor.sqrt())
        } else {
            (1.0, 1.0)
        };

        for child in children.iter() {
            let Ok((mut transform, sprite)) = sprite_query.get_mut(child) else {
                continue;
            };

            let mesh_half = capsule_query
                .get(child)
                .map(|c| c.half_length + 0.5)
                .unwrap_or(0.5);

            transform.scale = Vec3::new(scale_xz, scale_y, scale_xz);
            transform.translation.y = sprite.elevation - mesh_half * (1.0 - scale_y);
        }
    }
}
