use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::balance::MobsBalance;
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

#[derive(Copy, Clone, Debug)]
pub struct WaveModifiers {
    pub hp_mult: f32,
    pub damage_mult: f32,
}

impl Default for WaveModifiers {
    fn default() -> Self {
        Self { hp_mult: 1.0, damage_mult: 1.0 }
    }
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

pub fn spawn_mob(
    commands: &mut Commands,
    kind: MobKind,
    pos: Vec2,
    mobs: &MobsBalance,
    calculators: &StatCalculators,
    wave_mods: WaveModifiers,
) -> Entity {
    match kind {
        MobKind::Ghost => ghost::spawn_ghost(commands, pos, &mobs.ghost, calculators, wave_mods),
        MobKind::Tower => tower::spawn_tower(commands, pos, &mobs.tower, calculators, wave_mods),
        MobKind::SlimeSmall => slime::spawn_slime_small(commands, pos, &mobs.slime_small, calculators, wave_mods),
        MobKind::Spinner => spinner::spawn_spinner(commands, pos, &mobs.spinner, calculators, wave_mods),
        MobKind::Jumper => jumper::spawn_jumper(commands, pos, &mobs.jumper, calculators, wave_mods),
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
    wave_mods: WaveModifiers,
) -> Entity {
    let mut all_mods: Vec<(Stat, ModifierKind, f32)> = base_stats.to_vec();
    all_mods.push((Stat::MaxLife, ModifierKind::More, wave_mods.hp_mult - 1.0));
    all_mods.push((Stat::PhysicalDamage, ModifierKind::More, wave_mods.damage_mult - 1.0));
    let (modifiers, dirty, computed) = build_stats(calculators, &all_mods);
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
