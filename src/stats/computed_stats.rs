use bevy::prelude::*;

use super::Stat;

#[derive(Component, Clone)]
pub struct ComputedStats {
    values: [f32; Stat::COUNT],
}

impl Default for ComputedStats {
    fn default() -> Self {
        Self {
            values: [0.0; Stat::COUNT],
        }
    }
}

impl ComputedStats {
    pub fn get(&self, stat: Stat) -> f32 {
        self.values[stat.index()]
    }

    pub fn set(&mut self, stat: Stat, value: f32) {
        self.values[stat.index()] = value;
    }
}
