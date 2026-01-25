use bevy::prelude::*;

#[derive(Component)]
pub struct UseAbilities(pub Vec<String>);

impl UseAbilities {
    pub fn new(abilities: Vec<String>) -> Self {
        Self(abilities)
    }
}
