use bevy::prelude::*;
use std::collections::HashSet;

use super::StatId;

#[derive(Component, Default, Reflect)]
pub struct DirtyStats {
    pub stats: HashSet<StatId>,
}

impl DirtyStats {
    pub fn mark(&mut self, stat: StatId) {
        self.stats.insert(stat);
    }

    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    pub fn clear(&mut self) {
        self.stats.clear();
    }
}
