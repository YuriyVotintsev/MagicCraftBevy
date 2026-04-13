use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::components::common::sprite::Sprite as SpriteComp;
use crate::actors::components::mob::lunge_movement::{LungeMovementState, LungePhase};
use crate::composite_scale::{ScaleLayerId, ScaleLayerRegistry, ScaleModifiers};
use crate::stats::{ComputedStats, Stat};

#[derive(Component)]
pub struct SquishWalkAnimation {
    pub amount: f32,
}

#[derive(Component, Default)]
pub struct SquishState {
    dir: Vec2,
}

#[derive(Resource)]
pub struct SquishScaleLayer(pub ScaleLayerId);

pub fn register_systems(app: &mut App) {
    app.add_systems(Startup, register_layer);
    app.add_systems(PostUpdate, (init, animate).chain());
}

fn register_layer(mut registry: ResMut<ScaleLayerRegistry>, mut commands: Commands) {
    commands.insert_resource(SquishScaleLayer(registry.register()));
}

fn init(mut commands: Commands, query: Query<Entity, Added<SquishWalkAnimation>>) {
    for entity in &query {
        commands.entity(entity).insert(SquishState::default());
    }
}

pub fn animate(
    layer: Res<SquishScaleLayer>,
    mut query: Query<(
        &SquishWalkAnimation,
        &SpriteComp,
        &mut Transform,
        &ChildOf,
        &mut SquishState,
        &mut ScaleModifiers,
    )>,
    parent_query: Query<(&LinearVelocity, &ComputedStats, Option<&LungeMovementState>)>,
) {
    for (anim, sprite, mut transform, child_of, mut state, mut modifiers) in &mut query {
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
                .map(|(_, stats, _)| stats.get(Stat::MovementSpeed))
                .unwrap_or_default();
            let t = if max > 0.0 { (speed / max).clamp(0.0, 1.0) } else { 0.0 };
            (anim.amount * t, 0.0)
        };

        let perp = 1.0 / (1.0 + s).sqrt();
        modifiers.set(layer.0, Vec3::new(1.0 + s, perp, perp));

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
