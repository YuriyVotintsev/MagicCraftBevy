use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::content::{RuneKind, Tier};
use super::hex::HexCoord;
use crate::stats::Stat;

pub const GRID_RADIUS: i32 = 3;
pub const SHOP_SLOTS: usize = 4;
pub const JOKER_SLOTS: usize = 6;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rune {
    pub id: u32,
    pub color: Color,
    pub tier: Tier,
    pub kind: Option<RuneKind>,
    pub cost: u32,
}

impl Rune {
    pub fn is_joker(&self) -> bool {
        self.kind.is_none()
    }
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
    pub rune_id: u32,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct GridCellView {
    pub coord: HexCoord,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct JokerSlotView {
    pub index: usize,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Dragging {
    pub rune: Rune,
    pub from: RuneSource,
    pub grab_offset: Vec2,
}

#[derive(Resource)]
pub struct ShopOffer {
    pub stubs: [Option<Rune>; SHOP_SLOTS],
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
    pub cells: HashMap<HexCoord, Rune>,
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
    pub stubs: [Option<Rune>; JOKER_SLOTS],
}

#[derive(Resource, Default)]
pub struct RerollState {
    pub cost: u32,
}

#[derive(Resource, Default)]
pub struct GridHighlights {
    pub center_entity: Option<Entity>,
    pub center_pos: Option<Vec2>,
    pub write_targets: HashSet<HexCoord>,
    pub write_sources: HashSet<HexCoord>,
    pub pattern_cells: HashSet<HexCoord>,
}

#[derive(Resource)]
pub struct IconAssets {
    by_stat: HashMap<Stat, Handle<Image>>,
}

impl IconAssets {
    pub fn new(by_stat: HashMap<Stat, Handle<Image>>) -> Self {
        Self { by_stat }
    }

    pub fn for_stat(&self, stat: Stat) -> Option<&Handle<Image>> {
        self.by_stat.get(&stat)
    }
}

pub fn can_place(is_joker: bool, target: RuneSource, grid: &RuneGrid) -> bool {
    match (is_joker, target) {
        (false, RuneSource::Grid(c)) => grid.is_unlocked(c),
        (false, RuneSource::Shop(_)) => true,
        (true, RuneSource::Joker(_)) => true,
        (true, RuneSource::Shop(_)) => true,
        _ => false,
    }
}
