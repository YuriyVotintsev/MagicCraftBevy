use bevy::prelude::*;
use std::collections::HashMap;

use super::StatId;

#[derive(Component, Default, Reflect, Clone)]
pub struct ComputedStats {
    values: HashMap<StatId, f32>,
}

impl ComputedStats {
    pub fn get(&self, stat: StatId) -> f32 {
        self.values.get(&stat).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, stat: StatId, value: f32) {
        self.values.insert(stat, value);
    }
}
