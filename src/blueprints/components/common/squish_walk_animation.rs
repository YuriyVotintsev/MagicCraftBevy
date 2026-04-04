use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::blueprints::components::common::sprite::Sprite as SpriteComp;
use crate::blueprints::components::mob::lunge_movement::{LungeMovementState, LungePhase};
use crate::stats::{ComputedStats, StatRegistry};

#[blueprint_component]
pub struct SquishWalkAnimation {
    #[raw(default = 0.35)]
    pub amount: ScalarExpr,
}

#[derive(Component, Default)]
pub struct SquishState {
    dir: Vec2,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(PostUpdate, (init, animate).chain());
}

fn init(mut commands: Commands, query: Query<Entity, Added<SquishWalkAnimation>>) {
    for entity in &query {
        commands.entity(entity).insert(SquishState::default());
    }
}

pub fn animate(
    stat_registry: Option<Res<StatRegistry>>,
    mut query: Query<(
        &SquishWalkAnimation,
        &SpriteComp,
        &mut Transform,
        &ChildOf,
        &mut SquishState,
    )>,
    parent_query: Query<(&LinearVelocity, &ComputedStats, Option<&LungeMovementState>)>,
) {
    let Some(stat_registry) = stat_registry else {
        return;
    };
    let speed_id = stat_registry.get("movement_speed");

    for (anim, sprite, mut transform, child_of, mut state) in &mut query {
        let (vel2d, lunge_state) = parent_query
            .get(child_of.parent())
            .ok()
            .map(|(vel, _stats, ls)| (crate::coord::to_2d(vel.0), ls))
            .unwrap_or((Vec2::ZERO, None));

        let speed = vel2d.length();
        if speed > 0.01 {
            state.dir = vel2d / speed;
        }

        let r = sprite.scale / 2.0;
        let dir = state.dir;

        let (s, offset) = if let Some(ls) = lunge_state {
            if ls.phase == LungePhase::Lunging && ls.duration > 0.0 {
                let tau = (ls.elapsed / ls.duration).clamp(0.0, 1.0);
                let pi_tau = std::f32::consts::PI * tau;
                let s = anim.amount * pi_tau.sin();
                (s, s * r * pi_tau.cos())
            } else {
                (0.0, 0.0)
            }
        } else {
            let max = parent_query
                .get(child_of.parent())
                .ok()
                .and_then(|(_, stats, _)| speed_id.map(|id| stats.get(id)))
                .unwrap_or_default();
            let t = if max > 0.0 { (speed / max).clamp(0.0, 1.0) } else { 0.0 };
            (anim.amount * t, 0.0)
        };

        let perp = 1.0 / (1.0 + s).sqrt();
        transform.scale = Vec3::new(1.0 + s, perp, perp);

        if dir != Vec2::ZERO {
            transform.rotation = Quat::from_rotation_y(dir.y.atan2(dir.x));

            let offset_2d = sprite.position + dir * offset;
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
