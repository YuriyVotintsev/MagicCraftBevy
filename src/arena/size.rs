use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CurrentArenaSize {
    pub width: f32,
    pub height: f32,
}

impl CurrentArenaSize {
    pub fn half_w(&self) -> f32 { self.width / 2.0 }
    pub fn half_h(&self) -> f32 { self.height / 2.0 }
}
