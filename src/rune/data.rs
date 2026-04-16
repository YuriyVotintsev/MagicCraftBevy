use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::hex::HexCoord;

pub const GRID_RADIUS: i32 = 3;
pub const SHOP_SLOTS: usize = 4;
pub const JOKER_SLOTS: usize = 6;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StubKind {
    Rune,
    Joker,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RuneStub {
    pub id: u32,
    pub color: Color,
    pub kind: StubKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RuneSource {
    Shop(usize),
    Grid(HexCoord),
    Joker(usize),
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RuneView {
    pub source: RuneSource,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct GridCellView {
    pub coord: HexCoord,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct ShopSlotView {
    pub index: usize,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct JokerSlotView {
    pub index: usize,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Dragging {
    pub stub: RuneStub,
    pub from: RuneSource,
    pub grab_offset: Vec2,
}

#[derive(Resource)]
pub struct ShopOffer {
    pub stubs: [Option<RuneStub>; SHOP_SLOTS],
    pub next_id: u32,
}

impl Default for ShopOffer {
    fn default() -> Self {
        Self {
            stubs: [None; SHOP_SLOTS],
            next_id: 1,
        }
    }
}

#[derive(Resource)]
pub struct RuneGrid {
    pub cells: HashMap<HexCoord, RuneStub>,
    pub unlocked: HashSet<HexCoord>,
}

impl RuneGrid {
    pub fn new_with_initial_unlocked() -> Self {
        let mut unlocked = HashSet::new();
        unlocked.insert(HexCoord::CENTER);
        for n in HexCoord::CENTER.neighbors() {
            unlocked.insert(n);
        }
        Self {
            cells: HashMap::new(),
            unlocked,
        }
    }

    pub fn is_unlocked(&self, c: HexCoord) -> bool {
        self.unlocked.contains(&c)
    }
}

impl Default for RuneGrid {
    fn default() -> Self {
        Self::new_with_initial_unlocked()
    }
}

#[derive(Resource, Default)]
pub struct JokerSlots {
    pub stubs: [Option<RuneStub>; JOKER_SLOTS],
}

pub fn can_place(stub_kind: StubKind, target: RuneSource, grid: &RuneGrid) -> bool {
    match (stub_kind, target) {
        (StubKind::Rune, RuneSource::Grid(c)) => grid.is_unlocked(c),
        (StubKind::Rune, RuneSource::Shop(_)) => true,
        (StubKind::Joker, RuneSource::Joker(_)) => true,
        (StubKind::Joker, RuneSource::Shop(_)) => true,
        _ => false,
    }
}
