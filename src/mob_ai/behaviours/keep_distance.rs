use avian2d::prelude::*;
use bevy::prelude::*;

use crate::player::Player;
use crate::stats::{ComputedStats, StatRegistry};

#[derive(Component)]
pub struct KeepDistance {
    pub min_distance: f32,
    pub max_distance: f32,
}

impl KeepDistance {
    pub fn new(min_distance: f32, max_distance: f32) -> Self {
        Self {
            min_distance,
            max_distance,
        }
    }
}

pub fn keep_distance_system(
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

        velocity.0 = if distance < keep_dist.min_distance {
            -to_player.normalize_or_zero() * speed
        } else if distance > keep_dist.max_distance {
            to_player.normalize_or_zero() * speed
        } else {
            Vec2::ZERO
        };
    }
}
