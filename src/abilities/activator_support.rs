use bevy::prelude::*;
use super::ids::AbilityId;

#[derive(Component)]
pub struct AbilityInstance {
    pub ability_id: AbilityId,
    pub owner: Entity,
}
