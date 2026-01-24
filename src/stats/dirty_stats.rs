use bevy::prelude::*;
use std::collections::HashSet;

use super::StatId;

#[derive(Component, Default)]
pub struct DirtyStats {
    pub stats: HashSet<StatId>,
}

impl DirtyStats {
    pub fn mark(&mut self, stat: StatId) {
        self.stats.insert(stat);
    }

    pub fn mark_all(&mut self, stats: impl IntoIterator<Item = StatId>) {
        self.stats.extend(stats);
    }

    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    pub fn clear(&mut self) {
        self.stats.clear();
    }
}
