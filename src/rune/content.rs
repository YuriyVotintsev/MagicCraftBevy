use crate::stats::Stat;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tier {
    Common,
    Rare,
}

impl Tier {
    pub const ALL: &'static [Tier] = &[Tier::Common, Tier::Rare];
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RuneKind {
    Spike,
    HeartStone,
}

pub struct RuneDef {
    pub tier: Tier,
    pub limit: Option<u32>,
    pub base_effect: (Stat, f32),
    pub palette_key: &'static str,
}

impl RuneKind {
    pub const ALL: &'static [RuneKind] = &[RuneKind::Spike, RuneKind::HeartStone];

    pub fn def(self) -> RuneDef {
        match self {
            RuneKind::Spike => RuneDef {
                tier: Tier::Common,
                limit: None,
                base_effect: (Stat::PhysicalDamageFlat, 1.0),
                palette_key: "ui_rune_common",
            },
            RuneKind::HeartStone => RuneDef {
                tier: Tier::Rare,
                limit: Some(3),
                base_effect: (Stat::MaxLifeFlat, 5.0),
                palette_key: "ui_rune_rare",
            },
        }
    }
}
