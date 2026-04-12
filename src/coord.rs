use bevy::prelude::*;

pub fn ground_pos(v: Vec2) -> Vec3 {
    Vec3::new(v.x, 0.0, -v.y)
}

pub fn to_2d(v: Vec3) -> Vec2 {
    Vec2::new(v.x, -v.z)
}

pub fn ground_vel(v: Vec2) -> Vec3 {
    Vec3::new(v.x, 0.0, -v.y)
}
