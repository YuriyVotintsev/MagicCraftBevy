use bevy::prelude::*;

use super::{ComputedStats, DirtyStats, Modifiers, StatCalculators};

pub fn recalculate_stats(
    calculators: Res<StatCalculators>,
    mut query: Query<(&Modifiers, &mut ComputedStats, &mut DirtyStats)>,
) {
    for (modifiers, mut computed, mut dirty) in &mut query {
        if dirty.is_empty() {
            continue;
        }
        calculators.recalculate(modifiers, &mut computed, &mut dirty);
    }
}

pub fn mark_dirty_on_modifier_change(
    calculators: Res<StatCalculators>,
    mut query: Query<(&Modifiers, &mut DirtyStats), Changed<Modifiers>>,
) {
    for (modifiers, mut dirty) in &mut query {
        for stat in modifiers.affected_stats() {
            calculators.invalidate(stat, &mut dirty);
        }
    }
}
