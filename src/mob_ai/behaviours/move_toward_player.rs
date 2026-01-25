use bevy::prelude::*;

use crate::player::Player;
use crate::stats::{ComputedStats, StatRegistry};

#[derive(Component, Default)]
pub struct MoveTowardPlayer;

pub fn move_toward_player_system(
    time: Res<Time>,
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&mut Transform, &ComputedStats), With<MoveTowardPlayer>>,
    player: Query<&Transform, (With<Player>, Without<MoveTowardPlayer>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let speed_id = stat_registry.get("movement_speed");

    for (mut transform, stats) in &mut query {
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or(100.0);

        let direction = (player_pos - transform.translation).truncate();
        if direction.length_squared() > 1.0 {
            let normalized = direction.normalize();
            transform.translation.x += normalized.x * speed * time.delta_secs();
            transform.translation.y += normalized.y * speed * time.delta_secs();
        }
    }
}
