use bevy::prelude::*;
use std::sync::Arc;

use crate::stats::ComputedStats;

#[derive(Component)]
pub struct OwnedBy {
    pub owner: Entity,
    pub stats_snapshot: Arc<ComputedStats>,
}

impl OwnedBy {
    pub fn new(owner: Entity, stats: &ComputedStats) -> Self {
        Self {
            owner,
            stats_snapshot: Arc::new(stats.clone()),
        }
    }

    pub fn from_arc(owner: Entity, stats_snapshot: Arc<ComputedStats>) -> Self {
        Self {
            owner,
            stats_snapshot,
        }
    }
}
