use bevy::prelude::*;

use super::effects::OrbitingMovement;

pub fn update_orbiting_positions(
    time: Res<Time>,
    owner_query: Query<&Transform, Without<OrbitingMovement>>,
    mut orb_query: Query<(&mut OrbitingMovement, &mut Transform)>,
) {
    for (mut orbiting, mut transform) in &mut orb_query {
        orbiting.current_angle += orbiting.angular_speed * time.delta_secs();

        if let Ok(owner_transform) = owner_query.get(orbiting.owner) {
            let offset = Vec2::new(
                orbiting.current_angle.cos() * orbiting.radius,
                orbiting.current_angle.sin() * orbiting.radius,
            );
            transform.translation = owner_transform.translation + offset.extend(0.0);
        }
    }
}

pub fn cleanup_orbiting_on_owner_despawn(
    mut commands: Commands,
    orb_query: Query<(Entity, &OrbitingMovement)>,
    owner_query: Query<&Transform>,
) {
    for (entity, orbiting) in &orb_query {
        if owner_query.get(orbiting.owner).is_err() {
            commands.entity(entity).despawn();
        }
    }
}
