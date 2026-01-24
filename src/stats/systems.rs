use bevy::prelude::*;

use super::{ComputedStats, DirtyStats, RawStats, StatCalculators};

pub fn recalculate_stats(
    calculators: Res<StatCalculators>,
    mut query: Query<(&RawStats, &mut ComputedStats, &mut DirtyStats)>,
) {
    for (raw, mut computed, mut dirty) in &mut query {
        if dirty.is_empty() {
            continue;
        }

        for &stat in calculators.calculation_order() {
            if !dirty.stats.contains(&stat) {
                continue;
            }

            let value = calculators.calculate(stat, raw, &computed);
            computed.set(stat, value);
        }

        dirty.clear();
    }
}
