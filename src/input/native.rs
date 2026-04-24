// Desktop (native) backend for `PlayerInputPlugin`: reads WASD + mouse cursor + LMB,
// writes to `PlayerIntent`. Aim direction is computed by ray-casting the mouse cursor
// onto the ground plane and subtracting the player position.

use bevy::prelude::*;

use super::PlayerIntent;
use crate::actors::Player;
use crate::schedule::GameSet;
use crate::wave::WavePhase;

pub fn build(app: &mut App) {
    app.add_systems(
        Update,
        gather
            .before(GameSet::Input)
            .run_if(in_state(WavePhase::Combat)),
    );
}

fn gather(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_query: Query<&Transform, With<Player>>,
    mut intent: ResMut<PlayerIntent>,
) {
    let mut raw_move = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        raw_move.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        raw_move.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        raw_move.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        raw_move.x += 1.0;
    }
    intent.move_dir = raw_move.normalize_or_zero();
    intent.fire = mouse.pressed(MouseButton::Left);

    let mut aim = Vec2::ZERO;
    'aim: {
        let Ok(window) = windows.single() else { break 'aim };
        let Ok((camera, camera_transform)) = camera_query.single() else { break 'aim };
        let Ok(player) = player_query.single() else { break 'aim };
        let Some(cursor_pos) = window.cursor_position() else { break 'aim };
        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
            break 'aim;
        };
        let Some(distance) =
            ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
        else {
            break 'aim;
        };
        let world_pos = ray.get_point(distance);
        aim = crate::coord::to_2d(world_pos - player.translation).normalize_or_zero();
    }
    intent.aim_dir = aim;
}
