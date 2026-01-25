use bevy::prelude::*;

use super::{ComputedStats, DirtyStats, Modifiers, StatCalculators};

pub fn recalculate_stats(
    calculators: Res<StatCalculators>,
    mut query: Query<(&Modifiers, &mut ComputedStats, &mut DirtyStats)>,
) {
    for (modifiers, mut computed, mut dirty) in &mut query {
        calculators.recalculate(modifiers, &mut computed, &mut dirty);
    }
}
