use avian3d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::movement::SelfMoving;
use crate::stats::{ComputedStats, StatRegistry};
#[blueprint_component]
pub struct MoveToward {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
}

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
    stat_registry: Res<StatRegistry>,
    mut query: Query<(Entity, &Transform, &mut LinearVelocity, &ComputedStats, &MoveToward)>,
    transforms: Query<&Transform, Without<MoveToward>>,
) {
    let speed_id = stat_registry.get("movement_speed");

    for (entity, transform, mut velocity, stats, move_toward) in &mut query {
        let Ok(target_transform) = transforms.get(move_toward.target) else {
            continue;
        };
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or_default();
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
