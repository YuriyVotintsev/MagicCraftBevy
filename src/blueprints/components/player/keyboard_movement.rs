use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::MovementLocked;
use crate::schedule::GameSet;
use crate::stats::{ComputedStats, StatRegistry};
use crate::wave::WavePhase;

#[blueprint_component]
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
    keyboard: Res<ButtonInput<KeyCode>>,
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut LinearVelocity, &ComputedStats), (With<KeyboardMovement>, Without<MovementLocked>)>,
) {
    for (mut velocity, stats) in &mut query {
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

        let speed = stat_registry
            .get("movement_speed")
            .map(|id| stats.get(id))
            .unwrap_or_default();

        velocity.0 = if direction != Vec2::ZERO {
            direction.normalize() * speed
        } else {
            Vec2::ZERO
        };
    }
}
