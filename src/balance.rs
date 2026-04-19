use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::rune::Tier;

#[derive(Asset, Resource, TypePath, Clone, Deserialize)]
pub struct GameBalance {
    pub wave: WaveBalance,
    pub arena: ArenaBalance,
    pub run: RunBalance,
    pub runes: RuneBalance,
}

#[derive(Clone, Deserialize)]
pub struct WaveBalance {
    pub safe_spawn_radius: f32,
}

#[derive(Clone, Deserialize)]
pub struct ArenaBalance {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Deserialize)]
pub struct RunBalance {
    pub coins_per_kill: u32,
    pub coin_attraction_duration: f32,
}

#[derive(Clone, Deserialize)]
pub struct RuneBalance {
    pub joker_probability: f32,
    pub tier_weights: TierWeights,
    pub reroll_base_cost: u32,
    pub reroll_cost_step: u32,
}

#[derive(Clone, Deserialize)]
pub struct TierWeights {
    pub common: u32,
    pub rare: u32,
}

impl TierWeights {
    pub fn for_tier(&self, tier: Tier) -> u32 {
        match tier {
            Tier::Common => self.common,
            Tier::Rare => self.rare,
        }
    }

    pub fn total(&self) -> u32 {
        self.common + self.rare
    }
}
