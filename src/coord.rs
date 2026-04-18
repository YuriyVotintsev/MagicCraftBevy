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

pub fn cursor_ground_pos(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    cursor_plane_pos(window, camera, camera_transform, 0.0)
}

pub fn cursor_plane_pos(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    y: f32,
) -> Option<Vec3> {
    let cursor = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    let distance = ray.intersect_plane(Vec3::Y * y, InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(distance))
}
