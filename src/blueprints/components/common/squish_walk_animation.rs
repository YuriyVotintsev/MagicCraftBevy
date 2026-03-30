use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::sprite::Sprite as SpriteComp;
use crate::stats::{ComputedStats, StatRegistry};

#[blueprint_component]
pub struct SquishWalkAnimation {
    #[raw(default = 0.35)]
    pub amount: ScalarExpr,
}

#[derive(Component, Default)]
pub struct SquishState {
    s: f32,
    ds: f32,
    dir: Vec2,
}

const STIFFNESS: f32 = 200.0;
const DAMPING: f32 = 12.0;

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, (init, animate).chain());
}

fn init(mut commands: Commands, query: Query<Entity, Added<SquishWalkAnimation>>) {
    for entity in &query {
        commands.entity(entity).insert(SquishState::default());
    }
}

pub fn animate(
    time: Res<Time>,
    stat_registry: Option<Res<StatRegistry>>,
    mut query: Query<(
        &SquishWalkAnimation,
        &SpriteComp,
        &mut Transform,
        &ChildOf,
        &mut SquishState,
    )>,
    parent_query: Query<(&LinearVelocity, &ComputedStats)>,
) {
    let Some(stat_registry) = stat_registry else {
        return;
    };
    let speed_id = stat_registry.get("movement_speed");
    let dt = time.delta_secs().min(0.05);

    for (anim, sprite, mut transform, child_of, mut state) in &mut query {
        let (vel_dir, t) = parent_query
            .get(child_of.parent())
            .ok()
            .and_then(|(vel, stats)| {
                let max = speed_id.map(|id| stats.get(id)).unwrap_or_default();
                if max > 0.0 {
                    let vel2d = crate::coord::to_2d(vel.0);
                    let speed = vel2d.length();
                    let t = (speed / max).clamp(0.0, 1.0);
                    let dir = if speed > 0.01 {
                        vel2d / speed
                    } else {
                        Vec2::ZERO
                    };
                    Some((dir, t))
                } else {
                    None
                }
            })
            .unwrap_or((Vec2::ZERO, 0.0));

        if vel_dir != Vec2::ZERO {
            state.dir = vel_dir;
        }

        let target = anim.amount * t;
        let accel = (target - state.s) * STIFFNESS - state.ds * DAMPING;
        state.ds += accel * dt;
        state.s += state.ds * dt;

        let s = state.s;
        let r = sprite.scale / 2.0;
        let dir = state.dir;

        transform.scale = Vec3::new(1.0 + s, 1.0 - s, 1.0);

        if dir != Vec2::ZERO {
            transform.rotation = Quat::from_rotation_y((-dir.y).atan2(dir.x));

            let y_extent =
                (((1.0 + s) * dir.y).powi(2) + ((1.0 - s) * dir.x).powi(2)).sqrt();

            let offset_2d = Vec2::new(
                sprite.position.x + dir.x * s * r,
                sprite.position.y + r * (y_extent - 1.0) + dir.y * s * r,
            );
            let ground = crate::coord::ground_pos(offset_2d);
            transform.translation.x = ground.x;
            transform.translation.y = 0.5;
            transform.translation.z = ground.z;
        } else {
            transform.rotation = Quat::IDENTITY;
            let ground = crate::coord::ground_pos(sprite.position);
            transform.translation.x = ground.x;
            transform.translation.y = 0.5;
            transform.translation.z = ground.z;
        }
    }
}
