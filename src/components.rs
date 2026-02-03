use bevy::prelude::*;

#[derive(Component)]
pub struct Lifetime {
    pub remaining: f32,
}

#[derive(Component)]
pub struct Growing {
    pub start_size: f32,
    pub end_size: f32,
    pub duration: f32,
    pub elapsed: f32,
}

pub fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.remaining -= dt;
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn tick_growing(time: Res<Time>, mut query: Query<(&mut Growing, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut growing, mut transform) in &mut query {
        if growing.duration <= 0.0 {
            transform.scale = Vec3::splat(growing.start_size / 2.0);
            continue;
        }
        growing.elapsed += dt;
        let t = (growing.elapsed / growing.duration).clamp(0.0, 1.0);
        let size = growing.start_size + (growing.end_size - growing.start_size) * t;
        transform.scale = Vec3::splat(size / 2.0);
    }
}
