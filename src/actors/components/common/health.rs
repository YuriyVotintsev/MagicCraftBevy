use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: f32,
}

pub fn register_systems(_app: &mut App) {
}
