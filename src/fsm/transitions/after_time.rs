use bevy::prelude::*;

use crate::fsm::events::StateTransition;

#[derive(Component)]
pub struct AfterTime {
    pub target: String,
    pub duration: f32,
    pub elapsed: f32,
}

impl AfterTime {
    pub fn new(target: String, duration: f32) -> Self {
        Self {
            target,
            duration,
            elapsed: 0.0,
        }
    }
}

pub fn after_time_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut AfterTime)>,
    mut events: MessageWriter<StateTransition>,
) {
    for (entity, mut after_time) in &mut query {
        after_time.elapsed += time.delta_secs();
        if after_time.elapsed >= after_time.duration {
            events.write(StateTransition {
                entity,
                to: after_time.target.clone(),
            });
        }
    }
}
