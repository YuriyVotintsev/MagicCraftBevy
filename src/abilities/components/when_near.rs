use bevy::prelude::*;
use magic_craft_macros::ability_component;

use crate::player::Player;

#[ability_component]
pub struct WhenNear {
    pub to: String,
    pub distance: ScalarExpr,
}

pub fn register_systems(app: &mut App) {
    app.add_systems(
        Update,
        when_near_system.in_set(crate::schedule::GameSet::MobAI),
    );
}

fn when_near_system(
    query: Query<(Entity, &WhenNear, &Transform)>,
    player: Query<&Transform, With<Player>>,
    mut events: MessageWriter<crate::abilities::state::StateTransition>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, when_near, transform) in &query {
        let dist = transform.translation.distance(player_pos);
        if dist < when_near.distance {
            events.write(crate::abilities::state::StateTransition {
                entity,
                to: when_near.to.clone(),
            });
        }
    }
}
