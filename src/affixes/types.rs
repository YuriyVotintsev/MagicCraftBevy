use rand::prelude::*;

use crate::stats::{ModifierDef, ModifierDefRaw, StatId, StatRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AffixId(pub u32);

pub struct AffixDef {
    pub tiers: Vec<ModifierDef>,
}

impl AffixDef {
    pub fn max_tier(&self) -> usize {
        self.tiers.len().saturating_sub(1)
    }

    pub fn roll_values(&self, tier: usize, rng: &mut impl Rng) -> Vec<(StatId, f32)> {
        let tier_def = &self.tiers[tier];
        tier_def
            .stats
            .iter()
            .map(|sr| match sr {
                StatRange::Fixed { stat, value } => (*stat, *value),
                StatRange::Range { stat, min, max } => {
                    if (*max - *min).abs() < f32::EPSILON {
                        (*stat, *min)
                    } else {
                        (*stat, rng.random_range(*min..=*max))
                    }
                }
            })
            .collect()
    }
}

#[derive(serde::Deserialize)]
pub struct AffixDefRaw {
    pub id: String,
    pub tiers: Vec<ModifierDefRaw>,
}

#[derive(Debug, Clone)]
pub struct Affix {
    pub affix_id: AffixId,
    pub tier: usize,
    pub values: Vec<(StatId, f32)>,
}
