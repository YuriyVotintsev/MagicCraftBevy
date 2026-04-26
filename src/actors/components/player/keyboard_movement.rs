use avian3d::prelude::*;
use bevy::prelude::*;

use super::super::visual::SelfMoving;
use crate::input::PlayerIntent;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::wave::CombatPhase;

#[derive(Component)]
pub struct MovementLocked;

#[derive(Component)]
pub struct KeyboardMovement;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        intent_movement_system
            .in_set(GameSet::Input)
            .run_if(in_state(CombatPhase::Running)),
    );
}

fn intent_movement_system(
    mut commands: Commands,
    intent: Res<PlayerIntent>,
    mut query: Query<
        (Entity, &mut LinearVelocity, &ComputedStats),
        (With<KeyboardMovement>, Without<MovementLocked>),
    >,
) {
    for (entity, mut velocity, stats) in &mut query {
        let speed = stats.final_of(Stat::MovementSpeed);
        let dir = intent.move_dir;
        if dir.length_squared() > 1e-6 {
            commands.entity(entity).insert(SelfMoving);
            velocity.0 = crate::coord::ground_vel(dir * speed);
        } else {
            commands.entity(entity).remove::<SelfMoving>();
            velocity.0 = Vec3::ZERO;
        }
    }
}
