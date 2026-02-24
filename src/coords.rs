use avian3d::prelude::*;
use bevy::prelude::*;

pub fn vec2_to_3d(v: Vec2) -> Vec3 {
    Vec3::new(v.x, 0.0, -v.y)
}

pub fn vec3_to_2d(v: Vec3) -> Vec2 {
    Vec2::new(v.x, -v.z)
}

pub const GAME_LOCKED_AXES: LockedAxes = LockedAxes::new()
    .lock_translation_y()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_rotation_z();

pub const COLLIDER_HALF_HEIGHT: f32 = 0.01;

pub const PROJECTILE_FLOOR_Y: f32 = 30.0;
