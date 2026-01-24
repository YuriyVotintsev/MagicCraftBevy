use super::{DirtyStats, RawStats, StatCalculators, StatId};

pub struct Modifiers;

impl Modifiers {
    pub fn add_flat(
        raw: &mut RawStats,
        dirty: &mut DirtyStats,
        calculators: &StatCalculators,
        stat: StatId,
        value: f32,
    ) {
        raw.add(stat, value, dirty, calculators);
    }

    pub fn add_increased(
        raw: &mut RawStats,
        dirty: &mut DirtyStats,
        calculators: &StatCalculators,
        stat: StatId,
        percent: f32,
    ) {
        raw.add(stat, percent, dirty, calculators);
    }

    pub fn add_more(
        raw: &mut RawStats,
        dirty: &mut DirtyStats,
        calculators: &StatCalculators,
        stat: StatId,
        percent: f32,
    ) {
        raw.multiply(stat, 1.0 + percent, dirty, calculators);
    }
}
