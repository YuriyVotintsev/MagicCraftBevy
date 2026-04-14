use bevy::prelude::*;

use crate::stats::{Stat, StatCalculators};

use super::balance::MobsBalance;
use super::kind::MobKind;
use super::{ghost, jumper, slime, spinner, tower};

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
