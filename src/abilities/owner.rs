use bevy::prelude::*;
use std::sync::Arc;

use crate::stats::ComputedStats;

#[derive(Component)]
#[allow(dead_code)]
pub struct OwnedBy {
    pub owner: Entity,
    pub stats_snapshot: Arc<ComputedStats>,
}

impl OwnedBy {
    pub fn from_arc(owner: Entity, stats_snapshot: Arc<ComputedStats>) -> Self {
        Self {
            owner,
            stats_snapshot,
        }
    }
}
