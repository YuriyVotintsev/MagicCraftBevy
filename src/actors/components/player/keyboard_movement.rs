use avian3d::prelude::*;
use bevy::prelude::*;

use crate::MovementLocked;
use crate::movement::SelfMoving;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, Stat};
use crate::wave::WavePhase;

#[derive(Component)]
pub struct KeyboardMovement;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        keyboard_movement_system
            .in_set(GameSet::Input)
            .run_if(in_state(WavePhase::Combat)),
    );
}

fn keyboard_movement_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut LinearVelocity, &ComputedStats), (With<KeyboardMovement>, Without<MovementLocked>)>,
) {
    for (entity, mut velocity, stats) in &mut query {
        let mut direction = Vec2::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        let speed = stats.get(Stat::MovementSpeed);

        velocity.0 = if direction != Vec2::ZERO {
            commands.entity(entity).insert(SelfMoving);
            crate::coord::ground_vel(direction.normalize() * speed)
        } else {
            commands.entity(entity).remove::<SelfMoving>();
            Vec3::ZERO
        };
    }
}
