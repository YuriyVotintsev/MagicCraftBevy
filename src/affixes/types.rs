use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AffixId(pub u32);

#[derive(serde::Deserialize, Clone, Copy)]
pub struct AffixTier {
    pub min: f32,
    pub max: f32,
}

pub struct AffixDef {
    pub name: String,
    pub stat: crate::stats::StatId,
    pub tiers: Vec<AffixTier>,
}

impl AffixDef {
    pub fn format_value(&self, value: f32) -> String {
        let display = if self.name.contains('%') {
            format!("{}", (value * 100.0).round() as i32)
        } else {
            format!("{}", value.round() as i32)
        };
        self.name.replace("{}", &display)
    }

    pub fn format_display(&self, affix: &Affix) -> String {
        format!("{} [T{}]", self.format_value(affix.value), affix.tier + 1)
    }

    pub fn format_number(&self, value: f32) -> String {
        if self.name.contains('%') {
            format!("{}", (value * 100.0).round() as i32)
        } else {
            format!("{}", value.round() as i32)
        }
    }

    pub fn name_parts(&self) -> (&str, &str) {
        self.name.split_once("{}").unwrap_or((&self.name, ""))
    }

    pub fn max_tier(&self) -> usize {
        self.tiers.len().saturating_sub(1)
    }

    pub fn roll_value(&self, tier: usize, rng: &mut impl Rng) -> f32 {
        let t = &self.tiers[tier];
        if (t.max - t.min).abs() < f32::EPSILON {
            t.min
        } else {
            rng.random_range(t.min..=t.max)
        }
    }
}

#[derive(serde::Deserialize)]
pub struct AffixDefRaw {
    pub id: String,
    pub name: String,
    pub stat: String,
    pub tiers: Vec<AffixTier>,
}

#[derive(Debug, Clone, Copy)]
pub struct Affix {
    pub affix_id: AffixId,
    pub tier: usize,
    pub value: f32,
}
