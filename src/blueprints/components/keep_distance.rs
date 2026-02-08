use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::blueprint_component;

use crate::stats::{ComputedStats, StatRegistry};
#[blueprint_component]
pub struct KeepDistance {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
    pub min: ScalarExpr,
    pub max: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        keep_distance_system.in_set(crate::schedule::GameSet::MobAI),
    );
    app.add_observer(|on: On<Remove, KeepDistance>, mut q: Query<&mut LinearVelocity>| {
        if let Ok(mut v) = q.get_mut(on.event_target()) { v.0 = Vec2::ZERO; }
    });
}

fn keep_distance_system(
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&Transform, &mut LinearVelocity, &ComputedStats, &KeepDistance)>,
    transforms: Query<&Transform, Without<KeepDistance>>,
) {
    let speed_id = stat_registry.get("movement_speed");

    for (transform, mut velocity, stats, keep_dist) in &mut query {
        let Ok(target_transform) = transforms.get(keep_dist.target) else {
            continue;
        };
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or(100.0);
        let to_target = (target_transform.translation - transform.translation).truncate();
        let distance = to_target.length();

        velocity.0 = if distance < keep_dist.min {
            -to_target.normalize_or_zero() * speed
        } else if distance > keep_dist.max {
            to_target.normalize_or_zero() * speed
        } else {
            Vec2::ZERO
        };
    }
}
