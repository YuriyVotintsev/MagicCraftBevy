use bevy::prelude::*;

#[derive(Message)]
pub struct PendingDamage {
    pub target: Entity,
    pub amount: f32,
    pub source: Option<Entity>,
}
