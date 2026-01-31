use std::sync::LazyLock;
use bevy::prelude::*;

use super::StatId;

pub static DEFAULT_STATS: LazyLock<ComputedStats> = LazyLock::new(ComputedStats::default);

#[derive(Component, Default, Clone)]
pub struct ComputedStats {
    values: Vec<f32>,
}

impl ComputedStats {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: vec![0.0; capacity],
        }
    }

    pub fn get(&self, stat: StatId) -> f32 {
        self.values.get(stat.0 as usize).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, stat: StatId, value: f32) {
        let idx = stat.0 as usize;
        if idx >= self.values.len() {
            self.values.resize(idx + 1, 0.0);
        }
        self.values[idx] = value;
    }

}
