use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::player::Player;
use crate::stats::{ComputedStats, StatRegistry};

#[ability_component]
pub struct KeepDistance {
    pub min: ScalarExpr,
    pub max: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        keep_distance_system.in_set(crate::schedule::GameSet::MobAI),
    );
}

fn keep_distance_system(
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&Transform, &mut LinearVelocity, &ComputedStats, &KeepDistance)>,
    player: Query<&Transform, (With<Player>, Without<KeepDistance>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;
    let speed_id = stat_registry.get("movement_speed");

    for (transform, mut velocity, stats, keep_dist) in &mut query {
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or(100.0);
        let to_player = (player_pos - transform.translation).truncate();
        let distance = to_player.length();

        velocity.0 = if distance < keep_dist.min {
            -to_player.normalize_or_zero() * speed
        } else if distance > keep_dist.max {
            to_player.normalize_or_zero() * speed
        } else {
            Vec2::ZERO
        };
    }
}
