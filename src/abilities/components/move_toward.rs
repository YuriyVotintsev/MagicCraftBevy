use avian2d::prelude::*;
use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::player::Player;
use crate::stats::{ComputedStats, StatRegistry};

#[ability_component]
pub struct MoveToward;

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        move_toward_system.in_set(crate::schedule::GameSet::MobAI),
    );
}

fn move_toward_system(
    stat_registry: Res<StatRegistry>,
    mut query: Query<(&Transform, &mut LinearVelocity, &ComputedStats), With<MoveToward>>,
    player: Query<&Transform, (With<Player>, Without<MoveToward>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;
    let speed_id = stat_registry.get("movement_speed");

    for (transform, mut velocity, stats) in &mut query {
        let speed = speed_id.map(|id| stats.get(id)).unwrap_or(100.0);
        let direction = (player_pos - transform.translation).truncate();

        velocity.0 = if direction.length_squared() > 1.0 {
            direction.normalize() * speed
        } else {
            Vec2::ZERO
        };
    }
}
