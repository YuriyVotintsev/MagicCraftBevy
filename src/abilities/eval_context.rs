use crate::stats::ComputedStats;
use super::context::TargetInfo;

pub struct EvalContext<'a> {
    pub caster: TargetInfo,
    pub source: TargetInfo,
    pub target: TargetInfo,
    pub stats: &'a ComputedStats,
    pub index: usize,
    pub count: usize,
}

impl<'a> EvalContext<'a> {
    pub fn stats_only(stats: &'a ComputedStats) -> Self {
        Self {
            caster: TargetInfo::EMPTY,
            source: TargetInfo::EMPTY,
            target: TargetInfo::EMPTY,
            stats,
            index: 0,
            count: 1,
        }
    }
}
