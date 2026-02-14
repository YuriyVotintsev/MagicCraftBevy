use bevy::prelude::*;

use crate::player::SpellSlot;

use super::types::Affix;

#[derive(Component, Clone, Default)]
pub struct Affixes {
    pub affixes: [Option<Affix>; 6],
}

#[derive(Component)]
pub struct SpellSlotTag(pub SpellSlot);

#[derive(Component)]
pub struct SlotOwner(pub Entity);
