use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actors::SpawnSource;
use crate::actors::components::common::movement::SelfMoving;
use crate::stats::{ComputedStats, Stat};

#[derive(Component)]
pub struct MoveToward {}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        move_toward_system.in_set(crate::schedule::GameSet::MobAI),
    );
    app.add_observer(|on: On<Remove, MoveToward>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec3::ZERO; }
    });
}

fn move_toward_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut LinearVelocity, &ComputedStats, &SpawnSource), (With<MoveToward>, Without<crate::wave::summoning::RiseFromGround>)>,
    transforms: Query<&Transform, Without<MoveToward>>,
) {
    for (entity, transform, mut velocity, stats, source) in &mut query {
        let Some(target_entity) = source.target.entity else {
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<SelfMoving>();
            continue;
        };
        let Ok(target_transform) = transforms.get(target_entity) else {
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<SelfMoving>();
            continue;
        };
        let speed = stats.get(Stat::MovementSpeed);
        let direction = crate::coord::to_2d(target_transform.translation - transform.translation);

        velocity.0 = if direction.length_squared() > 1.0 {
            commands.entity(entity).insert(SelfMoving);
            crate::coord::ground_vel(direction.normalize() * speed)
        } else {
            commands.entity(entity).remove::<SelfMoving>();
            Vec3::ZERO
        };
    }
}
