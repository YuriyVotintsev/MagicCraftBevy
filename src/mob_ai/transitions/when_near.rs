use bevy::prelude::*;

use crate::fsm::StateTransition;
use crate::player::Player;

#[derive(Component)]
pub struct WhenNear(pub Vec<(String, f32)>);

impl WhenNear {
    pub fn new(transitions: Vec<(String, f32)>) -> Self {
        Self(transitions)
    }
}

pub fn when_near_system(
    query: Query<(Entity, &WhenNear, &Transform)>,
    player: Query<&Transform, With<Player>>,
    mut events: MessageWriter<StateTransition>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, when_near, transform) in &query {
        let dist = transform.translation.distance(player_pos);
        for (state, threshold) in &when_near.0 {
            if dist < *threshold {
                events.write(StateTransition {
                    entity,
                    to: state.clone(),
                });
                break;
            }
        }
    }
}
