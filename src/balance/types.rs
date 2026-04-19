use std::collections::HashMap;

use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::actors::MobKind;
use crate::rune::{RuneKind, Tier};

#[derive(Debug, Clone)]
pub struct MobCommonStats {
    pub hp: f32,
    pub damage: f32,
    pub speed: Option<f32>,
    pub size: f32,
    pub mass: Option<f32>,
    pub attack_speed: Option<f32>,
}

#[derive(Debug, Clone, Resource)]
pub struct MobsBalance {
    pub ghost: MobCommonStats,
    pub tower: MobCommonStats,
    pub slime_small: MobCommonStats,
    pub jumper: MobCommonStats,
    pub spinner: MobCommonStats,
}

impl MobsBalance {
    pub fn get(&self, kind: MobKind) -> &MobCommonStats {
        match kind {
            MobKind::Ghost => &self.ghost,
            MobKind::Tower => &self.tower,
            MobKind::SlimeSmall => &self.slime_small,
            MobKind::Spinner => &self.spinner,
            MobKind::Jumper => &self.jumper,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WaveDef {
    pub enemy_variety: u32,
    pub max_concurrent: u32,
    pub hp_multiplier: f32,
    pub damage_multiplier: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct WavesConfig {
    pub mob_unlocks: HashMap<MobKind, u32>,
    pub waves: Vec<WaveDef>,
}

impl WavesConfig {
    pub fn for_wave(&self, wave: u32) -> &WaveDef {
        let idx = (wave.saturating_sub(1) as usize).min(self.waves.len().saturating_sub(1));
        &self.waves[idx]
    }

    pub fn unlock_wave(&self, kind: MobKind) -> u32 {
        self.mob_unlocks.get(&kind).copied().unwrap_or(0)
    }

    pub fn resolve_pool(&self, wave: u32, rng: &mut impl Rng) -> Vec<MobKind> {
        let unlocked: Vec<MobKind> = MobKind::iter()
            .filter(|k| {
                let u = self.unlock_wave(*k);
                u > 0 && u <= wave
            })
            .collect();
        let mut picked: Vec<MobKind> = unlocked
            .iter()
            .copied()
            .filter(|k| {
                let u = self.unlock_wave(*k);
                u == wave || u + 1 == wave
            })
            .collect();
        let variety = self.for_wave(wave).enemy_variety as usize;
        let mut remaining: Vec<MobKind> =
            unlocked.into_iter().filter(|k| !picked.contains(k)).collect();
        remaining.shuffle(rng);
        let need = variety.saturating_sub(picked.len());
        picked.extend(remaining.into_iter().take(need));
        picked
    }
}

#[derive(Debug, Clone, Resource)]
pub struct RuneCosts {
    pub spike: u32,
    pub heart_stone: u32,
    pub resonator: u32,
}

impl RuneCosts {
    pub fn cost_for(&self, kind: RuneKind) -> u32 {
        match kind {
            RuneKind::Spike => self.spike,
            RuneKind::HeartStone => self.heart_stone,
            RuneKind::Resonator => self.resonator,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct Globals {
    pub safe_spawn_radius: f32,
    pub arena_radius: f32,
    pub coins_per_kill: u32,
    pub coin_attraction_duration: f32,
    pub rune_joker_probability: f32,
    pub rune_tier_weight_common: u32,
    pub rune_tier_weight_rare: u32,
    pub rune_reroll_base_cost: u32,
    pub rune_reroll_cost_step: u32,
}

impl Globals {
    pub fn rune_tier_weight(&self, tier: Tier) -> u32 {
        match tier {
            Tier::Common => self.rune_tier_weight_common,
            Tier::Rare => self.rune_tier_weight_rare,
        }
    }

    pub fn rune_tier_weight_total(&self) -> u32 {
        self.rune_tier_weight_common + self.rune_tier_weight_rare
    }
}

#[derive(Debug, Clone, Resource)]
pub struct Balance {
    pub mobs: MobsBalance,
    pub waves: WavesConfig,
    pub rune_costs: RuneCosts,
    pub globals: Globals,
}
