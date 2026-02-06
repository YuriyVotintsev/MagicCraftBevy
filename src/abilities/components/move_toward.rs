use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::stats::{ComputedStats, StatRegistry};

#[ability_component]
pub struct MoveToward {
    #[default_expr("target.entity")]
    pub target: EntityExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        move_toward_system.in_set(crate::schedule::GameSet::MobAI),
    );
}

fn move_toward_system(
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&Transform, &mut LinearVelocity, &ComputedStats, &MoveToward)>,
    transforms: Query<&Transform, Without<MoveToward>>,
) {
    let speed_id = stat_registry.get("movement_speed");

    for (transform, mut velocity, stats, move_toward) in &mut query {
        let Ok(target_transform) = transforms.get(move_toward.target) else {
            continue;
        };
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or(100.0);
        let direction = (target_transform.translation - transform.translation).truncate();

        velocity.0 = if direction.length_squared() > 1.0 {
            direction.normalize() * speed
        } else {
            Vec2::ZERO
        };
    }
}
