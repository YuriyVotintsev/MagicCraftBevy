use std::collections::HashMap;

use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::actors::MobKind;

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
pub struct Globals {
    pub safe_spawn_radius: f32,
    pub arena_radius: f32,
}

#[derive(Debug, Clone, Resource)]
pub struct Balance {
    pub mobs: MobsBalance,
    pub waves: WavesConfig,
    pub globals: Globals,
}
