use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;
use strum::IntoEnumIterator;

use crate::faction::Faction;
use crate::palette;
use crate::stats::{ComputedStats, DirtyStats, ModifierKind, Modifiers, Stat, StatCalculators};

use super::super::components::{
    Caster, Collider, ColliderShape, DynamicBody, Health, OnDeathParticles, Shadow, ShapeColor,
    Size, StaticBody,
};
use super::{ghost, jumper, slime, spinner, tower};

#[derive(
    Copy, Clone, Debug, Hash, Eq, PartialEq,
    strum::EnumIter, strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum MobKind {
    Ghost,
    Tower,
    SlimeSmall,
    Spinner,
    Jumper,
}

impl MobKind {
    pub fn id(self) -> &'static str {
        self.into()
    }

    pub fn iter() -> impl Iterator<Item = MobKind> {
        <Self as IntoEnumIterator>::iter()
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
) -> Entity {
    match kind {
        MobKind::Ghost => ghost::spawn_ghost(commands, pos, &mobs.ghost, calculators),
        MobKind::Tower => tower::spawn_tower(commands, pos, &mobs.tower, calculators),
        MobKind::SlimeSmall => slime::spawn_slime_small(commands, pos, &mobs.slime_small, calculators),
        MobKind::Spinner => spinner::spawn_spinner(commands, pos, &mobs.spinner, calculators),
        MobKind::Jumper => jumper::spawn_jumper(commands, pos, &mobs.jumper, calculators),
    }
}

pub(super) fn enemy_shape_color() -> ShapeColor {
    let (r, g, b) = palette::lookup("enemy").unwrap_or((1.0, 1.0, 1.0));
    let flash = palette::flash_lookup("enemy");
    ShapeColor { r, g, b, a: 1.0, flash }
}

pub(super) fn enemy_ability_shape_color() -> ShapeColor {
    let (r, g, b) = palette::lookup("enemy_ability").unwrap_or((1.0, 0.5, 0.5));
    let flash = palette::flash_lookup("enemy_ability");
    ShapeColor { r, g, b, a: 1.0, flash }
}

pub(crate) enum EnemyBody {
    Dynamic { mass: f32 },
    Static,
}

pub(crate) fn spawn_enemy_core(
    commands: &mut Commands,
    pos: Vec2,
    calculators: &StatCalculators,
    base_stats: &[(Stat, ModifierKind, f32)],
    size: f32,
    body: EnemyBody,
    death_particles: &'static str,
) -> Entity {
    let (modifiers, dirty, computed) = build_stats(calculators, base_stats);
    let hp = computed.final_of(Stat::MaxLife);
    let ground = crate::coord::ground_pos(pos);

    let id = commands.spawn_empty().id();
    commands.entity(id).insert((
        Transform::from_translation(ground),
        Visibility::default(),
        Faction::Enemy,
        modifiers, dirty, computed,
        Size { value: size },
        Collider { shape: ColliderShape::Circle, sensor: false },
        Health { current: hp },
        Caster(id),
        OnDeathParticles { config: death_particles },
    ));

    match body {
        EnemyBody::Dynamic { mass } => {
            commands.entity(id).insert(DynamicBody { mass });
        }
        EnemyBody::Static => {
            commands.entity(id).insert(StaticBody);
        }
    }

    commands.entity(id).with_children(|p| {
        p.spawn(Shadow);
    });

    id
}

fn build_stats(
    calculators: &StatCalculators,
    base_stats: &[(Stat, ModifierKind, f32)],
) -> (Modifiers, DirtyStats, ComputedStats) {
    let mut modifiers = Modifiers::new();
    for &(stat, kind, value) in base_stats {
        modifiers.add(stat, kind, value);
    }
    let mut dirty = DirtyStats::default();
    let mut computed = ComputedStats::default();
    dirty.mark_all(Stat::iter());
    calculators.recalculate(&modifiers, &mut computed, &mut dirty);
    (modifiers, dirty, computed)
}
