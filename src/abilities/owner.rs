use bevy::prelude::*;

#[derive(Component)]
pub struct OwnedBy {
    #[allow(dead_code)]
    pub owner: Entity,
}

impl OwnedBy {
    pub fn new(owner: Entity) -> Self {
        Self { owner }
    }
}
