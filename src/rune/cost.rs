use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use super::content::RuneKind;

#[derive(Asset, Resource, TypePath, Clone, Deserialize)]
pub struct RuneCosts {
    pub spike: u32,
    pub heart_stone: u32,
}

impl RuneCosts {
    pub fn cost_for(&self, kind: RuneKind) -> u32 {
        match kind {
            RuneKind::Spike => self.spike,
            RuneKind::HeartStone => self.heart_stone,
        }
    }
}
