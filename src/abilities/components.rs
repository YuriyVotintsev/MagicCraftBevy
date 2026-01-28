use bevy::prelude::*;

pub use super::ids::AbilityId;

#[derive(Clone, Default)]
pub struct HeldAbility {
    pub ability_id: AbilityId,
    pub target_direction: Vec3,
    pub target_point: Vec3,
}

#[derive(Component, Default)]
pub struct AbilityInput {
    pub want_to_cast: Option<AbilityId>,
    pub target_direction: Option<Vec3>,
    pub target_point: Option<Vec3>,
    pub holding: Option<HeldAbility>,
}

impl AbilityInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.want_to_cast = None;
        self.target_direction = None;
        self.target_point = None;
    }
}
