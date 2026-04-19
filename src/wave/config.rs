use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::Deserialize;

use crate::actors::MobKind;

#[derive(Asset, Resource, TypePath, Clone, Deserialize)]
pub struct WavesConfig {
    pub mob_unlocks: MobUnlocks,
    pub waves: Vec<WaveDef>,
}

#[derive(Clone, Deserialize)]
pub struct MobUnlocks {
    pub ghost: u32,
    pub tower: u32,
    pub slime_small: u32,
    pub spinner: u32,
    pub jumper: u32,
}

#[derive(Clone, Deserialize)]
pub struct WaveDef {
    pub enemy_variety: u32,
    pub max_concurrent: u32,
    pub hp_multiplier: f32,
    pub damage_multiplier: f32,
}

impl WavesConfig {
    pub fn for_wave(&self, wave: u32) -> &WaveDef {
        let idx = (wave.saturating_sub(1) as usize).min(self.waves.len().saturating_sub(1));
        &self.waves[idx]
    }

    pub fn unlock_wave(&self, kind: MobKind) -> u32 {
        match kind {
            MobKind::Ghost => self.mob_unlocks.ghost,
            MobKind::Tower => self.mob_unlocks.tower,
            MobKind::SlimeSmall => self.mob_unlocks.slime_small,
            MobKind::Spinner => self.mob_unlocks.spinner,
            MobKind::Jumper => self.mob_unlocks.jumper,
        }
    }

    pub fn resolve_pool(&self, wave: u32, rng: &mut impl Rng) -> Vec<MobKind> {
        let unlocked: Vec<MobKind> = MobKind::iter()
            .filter(|k| self.unlock_wave(*k) <= wave && self.unlock_wave(*k) > 0)
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
