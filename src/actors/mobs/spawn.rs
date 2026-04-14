use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::palette;
use crate::stats::{ComputedStats, DirtyStats, Modifiers, Stat, StatCalculators};

use super::super::components::SpriteColor;
use super::{ghost, jumper, slime, spinner, tower};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MobKind {
    Ghost,
    Tower,
    SlimeSmall,
    Spinner,
    Jumper,
}

impl MobKind {
    #[allow(dead_code)]
    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "ghost" => Some(MobKind::Ghost),
            "tower" => Some(MobKind::Tower),
            "slime_small" => Some(MobKind::SlimeSmall),
            "spinner" => Some(MobKind::Spinner),
            "jumper" => Some(MobKind::Jumper),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            MobKind::Ghost => "ghost",
            MobKind::Tower => "tower",
            MobKind::SlimeSmall => "slime_small",
            MobKind::Spinner => "spinner",
            MobKind::Jumper => "jumper",
        }
    }

    pub fn size(&self, mobs: &MobsBalance) -> f32 {
        match self {
            MobKind::Ghost => mobs.ghost.size,
            MobKind::Tower => mobs.tower.size,
            MobKind::SlimeSmall => mobs.slime_small.size,
            MobKind::Spinner => mobs.spinner.size,
            MobKind::Jumper => mobs.jumper.size,
        }
    }
}

#[derive(Asset, Resource, TypePath, Clone, Deserialize, Debug)]
pub struct MobsBalance {
    pub ghost: ghost::GhostStats,
    pub tower: tower::TowerStats,
    pub slime_small: slime::SlimeSmallStats,
    pub jumper: jumper::JumperStats,
    pub spinner: spinner::SpinnerStats,
}

pub fn spawn_mob(
    commands: &mut Commands,
    kind: MobKind,
    pos: Vec2,
    mobs: &MobsBalance,
    calculators: &StatCalculators,
    extra_modifiers: &[(Stat, f32)],
) -> Entity {
    match kind {
        MobKind::Ghost => ghost::spawn_ghost(commands, pos, &mobs.ghost, calculators, extra_modifiers),
        MobKind::Tower => tower::spawn_tower(commands, pos, &mobs.tower, calculators, extra_modifiers),
        MobKind::SlimeSmall => slime::spawn_slime_small(commands, pos, &mobs.slime_small, calculators, extra_modifiers),
        MobKind::Spinner => spinner::spawn_spinner(commands, pos, &mobs.spinner, calculators, extra_modifiers),
        MobKind::Jumper => jumper::spawn_jumper(commands, pos, &mobs.jumper, calculators, extra_modifiers),
    }
}

pub(super) fn enemy_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy").unwrap_or((1.0, 1.0, 1.0));
    let flash = palette::flash_lookup("enemy");
    SpriteColor { r, g, b, a: 1.0, flash }
}

pub(super) fn enemy_ability_sprite_color() -> SpriteColor {
    let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
    let flash = palette::flash_lookup("enemy_ability");
    SpriteColor { r, g, b, a: 1.0, flash }
}

pub(super) fn compute_stats(
    calculators: &StatCalculators,
    base_stats: &[(Stat, f32)],
    extra_modifiers: &[(Stat, f32)],
) -> (Modifiers, DirtyStats, ComputedStats) {
    let mut modifiers = Modifiers::new();
    for &(stat, value) in base_stats {
        modifiers.add(stat, value);
    }
    for &(stat, value) in extra_modifiers {
        modifiers.add(stat, value);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::ALL.iter().copied());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    (modifiers, dirty, computed)
}

pub(super) fn current_max_life(computed: &ComputedStats) -> f32 {
    computed.get(Stat::MaxLife)
}
