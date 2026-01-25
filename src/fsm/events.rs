use bevy::prelude::*;

#[derive(Message)]
pub struct StateTransition {
    pub entity: Entity,
    pub to: String,
}
