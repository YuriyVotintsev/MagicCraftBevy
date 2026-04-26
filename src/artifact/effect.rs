use bevy::prelude::*;

use crate::stats::{ComputedStats, ModifierKind, Stat};

#[derive(Copy, Clone, Debug)]
pub enum ArtifactEffect {
    StatMod {
        stat: Stat,
        kind: ModifierKind,
        value: f32,
    },
    Multishot {
        extra: u32,
    },
    Pierce {
        extra: u32,
    },
    Ricochet {
        count: u32,
    },
    Homing {
        strength: f32,
    },
    Splash {
        radius: f32,
    },
    OnHit(OnHitKind),
    Defensive(DefensiveKind),
    Exotic(ExoticKind),
}

#[derive(Copy, Clone, Debug)]
pub enum OnHitKind {
    Burn { dps: f32, duration: f32 },
    Freeze { chance: f32, duration: f32 },
    Lifesteal { pct: f32 },
    Knockback { force: f32 },
    Chain { count: u32 },
}

#[derive(Copy, Clone, Debug)]
pub enum DefensiveKind {
    Shield { max_block: f32, recharge: f32 },
    Dodge { chance: f32 },
    Thorns { reflect_pct: f32 },
}

#[derive(Copy, Clone, Debug)]
pub enum ExoticKind {
    Turret { fire_interval: f32, damage_pct: f32 },
    OrbitingOrbs { count: u32, radius: f32, damage: f32 },
    PeriodicAoe { interval: f32, radius: f32, damage_pct: f32 },
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct OnHitEffectStack {
    pub burn: Option<(f32, f32)>,
    pub freeze: Option<(f32, f32)>,
    pub lifesteal_pct: f32,
    pub knockback: f32,
    pub chain: Option<u32>,
}

impl OnHitEffectStack {
    pub fn is_empty(self) -> bool {
        self.burn.is_none()
            && self.freeze.is_none()
            && self.lifesteal_pct == 0.0
            && self.knockback == 0.0
            && self.chain.is_none()
    }

    pub fn from_stats(stats: Option<&ComputedStats>) -> Self {
        let Some(s) = stats else { return Self::default(); };
        let burn_dps = s.final_of(Stat::BurnDPS);
        let burn_dur = s.final_of(Stat::BurnDuration);
        let burn = if burn_dps > 0.0 && burn_dur > 0.0 {
            Some((burn_dps, burn_dur))
        } else {
            None
        };

        let freeze_chance = s.final_of(Stat::FreezeChance);
        let freeze_dur = s.final_of(Stat::FreezeDuration);
        let freeze = if freeze_chance > 0.0 && freeze_dur > 0.0 {
            Some((freeze_chance, freeze_dur))
        } else {
            None
        };

        let chain_count = s.final_of(Stat::ChainCount).max(0.0) as u32;
        let chain = if chain_count > 0 { Some(chain_count) } else { None };

        Self {
            burn,
            freeze,
            lifesteal_pct: s.final_of(Stat::Lifesteal),
            knockback: s.final_of(Stat::KnockbackForce),
            chain,
        }
    }
}
