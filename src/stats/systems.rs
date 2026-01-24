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

        for &stat in calculators.calculation_order() {
            if !dirty.stats.contains(&stat) {
                continue;
            }

            let value = calculators.calculate(stat, modifiers, &computed);
            computed.set(stat, value);
        }

        dirty.clear();
    }
}
