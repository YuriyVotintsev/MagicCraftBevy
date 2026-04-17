use super::data::RuneGrid;
use super::hex::HexCoord;
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
    Resonator,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    Adjacent,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Filter {
    Any,
}

#[derive(Copy, Clone, Debug)]
pub enum WriteEffect {
    More { factor: f32 },
}

#[derive(Copy, Clone, Debug)]
pub struct Write {
    pub pattern: Pattern,
    pub filter: Filter,
    pub effect: WriteEffect,
}

pub struct RuneDef {
    pub tier: Tier,
    pub limit: Option<u32>,
    pub base_effect: (Stat, ModifierKind, f32),
    pub write: Option<Write>,
    pub palette_key: &'static str,
}

impl RuneKind {
    pub const ALL: &'static [RuneKind] =
        &[RuneKind::Spike, RuneKind::HeartStone, RuneKind::Resonator];

    pub fn def(self) -> RuneDef {
        match self {
            RuneKind::Spike => RuneDef {
                tier: Tier::Common,
                limit: None,
                base_effect: (Stat::PhysicalDamage, ModifierKind::Flat, 1.0),
                write: None,
                palette_key: "ui_rune_common",
            },
            RuneKind::HeartStone => RuneDef {
                tier: Tier::Rare,
                limit: Some(3),
                base_effect: (Stat::MaxLife, ModifierKind::Flat, 5.0),
                write: None,
                palette_key: "ui_rune_rare",
            },
            RuneKind::Resonator => RuneDef {
                tier: Tier::Rare,
                limit: Some(2),
                base_effect: (Stat::AttackSpeed, ModifierKind::More, 0.10),
                write: Some(Write {
                    pattern: Pattern::Adjacent,
                    filter: Filter::Any,
                    effect: WriteEffect::More { factor: 1.5 },
                }),
                palette_key: "ui_rune_rare",
            },
        }
    }
}

pub fn write_pattern_contains(write: &Write, src: HexCoord, target: HexCoord) -> bool {
    match (write.pattern, write.filter) {
        (Pattern::Adjacent, Filter::Any) => src.neighbors().contains(&target),
    }
}

pub fn write_targets(write: &Write, center: HexCoord, grid: &RuneGrid) -> Vec<HexCoord> {
    match (write.pattern, write.filter) {
        (Pattern::Adjacent, Filter::Any) => center
            .neighbors()
            .into_iter()
            .filter(|c| grid.cells.contains_key(c))
            .collect(),
    }
}

pub fn write_pattern_coords(write: &Write, center: HexCoord) -> Vec<HexCoord> {
    match write.pattern {
        Pattern::Adjacent => center.neighbors().into_iter().collect(),
    }
}
