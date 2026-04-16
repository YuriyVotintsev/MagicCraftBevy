use crate::stats::{ModifierKind, Stat};

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
    pub base_effect: (Stat, ModifierKind, f32),
    pub palette_key: &'static str,
}

impl RuneKind {
    pub const ALL: &'static [RuneKind] = &[RuneKind::Spike, RuneKind::HeartStone];

    pub fn def(self) -> RuneDef {
        match self {
            RuneKind::Spike => RuneDef {
                tier: Tier::Common,
                limit: None,
                base_effect: (Stat::PhysicalDamage, ModifierKind::Flat, 1.0),
                palette_key: "ui_rune_common",
            },
            RuneKind::HeartStone => RuneDef {
                tier: Tier::Rare,
                limit: Some(3),
                base_effect: (Stat::MaxLife, ModifierKind::Flat, 5.0),
                palette_key: "ui_rune_rare",
            },
        }
    }
}
