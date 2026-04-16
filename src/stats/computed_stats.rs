use bevy::prelude::*;

use super::registry::{Formula, ModifierKind, Stat};

#[derive(Component, Clone)]
pub struct ComputedStats {
    buckets: [[f32; ModifierKind::COUNT]; Stat::COUNT],
    finals: [f32; Stat::COUNT],
}

impl Default for ComputedStats {
    fn default() -> Self {
        let mut buckets = [[0.0; ModifierKind::COUNT]; Stat::COUNT];
        for row in buckets.iter_mut() {
            row[ModifierKind::More.index()] = 1.0;
        }
        Self {
            buckets,
            finals: [0.0; Stat::COUNT],
        }
    }
}

impl ComputedStats {
    pub fn bucket(&self, stat: Stat, kind: ModifierKind) -> f32 {
        self.buckets[stat.index()][kind.index()]
    }

    pub fn set_bucket(&mut self, stat: Stat, kind: ModifierKind, value: f32) {
        self.buckets[stat.index()][kind.index()] = value;
    }

    pub fn set_final(&mut self, stat: Stat, value: f32) {
        self.finals[stat.index()] = value;
    }

    pub fn final_of(&self, stat: Stat) -> f32 {
        self.finals[stat.index()]
    }

    pub fn apply(&self, stat: Stat, base: f32) -> f32 {
        match stat.formula() {
            Formula::FlatIncMore => {
                let flat = self.bucket(stat, ModifierKind::Flat);
                let inc = self.bucket(stat, ModifierKind::Increased);
                let more = self.bucket(stat, ModifierKind::More);
                (base + flat) * (1.0 + inc) * more
            }
            Formula::Custom(f) => f(self, stat, base),
        }
    }
}
