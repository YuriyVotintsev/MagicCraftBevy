use bevy::prelude::*;

use crate::stats::ComputedStats;
use super::components::ComponentDef;
use super::AbilitySource;

#[derive(Component)]
pub struct StoredComponentDefs {
    pub base: Vec<ComponentDef>,
    pub state: Vec<ComponentDef>,
}

impl StoredComponentDefs {
    pub fn all(&self) -> impl Iterator<Item = &ComponentDef> {
        self.base.iter().chain(self.state.iter())
    }
}

pub fn recalculate_on_stats_change(
    mut commands: Commands,
    query: Query<(Entity, &AbilitySource, &ComputedStats, &StoredComponentDefs), Changed<ComputedStats>>,
) {
    for (entity, source, stats, defs) in &query {
        for def in defs.all() {
            def.update_component(&mut commands.entity(entity), source, stats);
        }
    }
}
