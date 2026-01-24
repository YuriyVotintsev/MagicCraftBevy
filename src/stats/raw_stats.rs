use bevy::prelude::*;
use std::collections::HashMap;

use super::{DirtyStats, StatCalculators, StatId};

#[derive(Component, Default, Reflect)]
pub struct RawStats {
    values: HashMap<StatId, f32>,
}

impl RawStats {
    pub fn get(&self, stat: StatId) -> f32 {
        self.values.get(&stat).copied().unwrap_or(0.0)
    }

    pub fn set(
        &mut self,
        stat: StatId,
        value: f32,
        dirty: &mut DirtyStats,
        calculators: &StatCalculators,
    ) {
        let old = self.values.insert(stat, value);
        if old != Some(value) {
            calculators.invalidate(stat, dirty);
        }
    }

    pub fn add(
        &mut self,
        stat: StatId,
        delta: f32,
        dirty: &mut DirtyStats,
        calculators: &StatCalculators,
    ) {
        let current = self.get(stat);
        self.set(stat, current + delta, dirty, calculators);
    }

    pub fn multiply(
        &mut self,
        stat: StatId,
        factor: f32,
        dirty: &mut DirtyStats,
        calculators: &StatCalculators,
    ) {
        let current = self.get(stat);
        if current == 0.0 {
            self.set(stat, factor, dirty, calculators);
        } else {
            self.set(stat, current * factor, dirty, calculators);
        }
    }
}
