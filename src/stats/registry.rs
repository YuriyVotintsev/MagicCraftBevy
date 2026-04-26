use bevy::prelude::*;
use strum::{EnumCount, IntoEnumIterator};

use super::ComputedStats;

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, Debug, Reflect,
    strum::EnumIter, strum::EnumCount, strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Stat {
    MaxLife,
    MaxMana,
    PhysicalDamage,
    MovementSpeed,
    ProjectileSpeed,
    ProjectileCount,
    CritChance,
    CritMultiplier,
    AttackSpeed,
    AreaOfEffect,
    Duration,
    PickupRadius,
    DodgeChance,
    Lifesteal,
    Thorns,
    KnockbackForce,
    Pierce,
    Ricochet,
    HomingStrength,
    SplashRadius,
    BurnDPS,
    BurnDuration,
    FreezeChance,
    FreezeDuration,
    ChainCount,
    ShieldMaxBlock,
    ShieldRecharge,
}

impl Stat {
    pub const COUNT: usize = <Self as EnumCount>::COUNT;

    pub fn iter() -> impl Iterator<Item = Stat> {
        <Self as IntoEnumIterator>::iter()
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        self.into()
    }

    pub fn formula(self) -> Formula {
        match self {
            Stat::CritChance | Stat::DodgeChance | Stat::FreezeChance => {
                Formula::Custom(clamped_chance)
            }
            _ => Formula::FlatIncMore,
        }
    }

    pub fn deps(self) -> &'static [Stat] {
        &[]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ModifierKind {
    Flat,
    Increased,
    More,
}

impl ModifierKind {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone)]
pub enum Formula {
    FlatIncMore,
    Custom(fn(&ComputedStats, Stat, f32) -> f32),
}

fn clamped_chance(cs: &ComputedStats, stat: Stat, base: f32) -> f32 {
    let flat = cs.bucket(stat, ModifierKind::Flat);
    let inc = cs.bucket(stat, ModifierKind::Increased);
    ((base + flat) * (1.0 + inc)).clamp(0.0, 1.0)
}
