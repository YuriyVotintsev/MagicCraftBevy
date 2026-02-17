use serde::Deserialize;

use super::{StatId, StatRegistry};

#[derive(Debug, Clone)]
pub enum StatRange {
    Fixed { stat: StatId, value: f32 },
    Range { stat: StatId, min: f32, max: f32 },
}

#[derive(Debug, Clone)]
pub struct ModifierDef {
    pub stats: Vec<StatRange>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum StatRangeRaw {
    Fixed { stat: String, value: f32 },
    Range { stat: String, min: f32, max: f32 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModifierDefRaw {
    pub stats: Vec<StatRangeRaw>,
}

impl ModifierDefRaw {
    pub fn resolve(&self, registry: &StatRegistry) -> ModifierDef {
        let stats = self
            .stats
            .iter()
            .filter_map(|sr| match sr {
                StatRangeRaw::Fixed { stat, value } => {
                    registry.get(stat).map(|id| StatRange::Fixed { stat: id, value: *value })
                }
                StatRangeRaw::Range { stat, min, max } => {
                    registry.get(stat).map(|id| StatRange::Range { stat: id, min: *min, max: *max })
                }
            })
            .collect();
        ModifierDef { stats }
    }
}
