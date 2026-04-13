use bevy::prelude::*;
use std::collections::HashSet;

use super::Stat;

#[derive(Component, Default)]
pub struct DirtyStats {
    pub stats: HashSet<Stat>,
}

impl DirtyStats {
    pub fn mark(&mut self, stat: Stat) {
        self.stats.insert(stat);
    }

    pub fn mark_all(&mut self, stats: impl IntoIterator<Item = Stat>) {
        self.stats.extend(stats);
    }

    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    pub fn clear(&mut self) {
        self.stats.clear();
    }
}
