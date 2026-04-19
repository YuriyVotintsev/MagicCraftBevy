use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::content::{RuneKind, Tier};
use super::hex::HexCoord;

pub const GRID_RADIUS: i32 = 2;
pub const SHOP_SLOTS: usize = 4;
pub const JOKER_SLOT_COUNT: usize = 6;

pub const JOKER_COORDS: [HexCoord; JOKER_SLOT_COUNT] = [
    HexCoord { q: 2, r: 0 },
    HexCoord { q: 2, r: -2 },
    HexCoord { q: 0, r: -2 },
    HexCoord { q: -2, r: 0 },
    HexCoord { q: -2, r: 2 },
    HexCoord { q: 0, r: 2 },
];

pub fn is_joker_slot(c: HexCoord) -> bool {
    JOKER_COORDS.iter().any(|j| j.q == c.q && j.r == c.r)
}

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
pub struct Dragging {
    pub rune: Rune,
    pub from: RuneSource,
    pub grab_offset: Vec3,
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

#[derive(Resource, Clone)]
pub struct RuneGrid {
    pub cells: HashMap<HexCoord, Rune>,
    pub unlocked: HashSet<HexCoord>,
}

impl RuneGrid {
    pub fn new_with_initial_unlocked() -> Self {
        let unlocked = HexCoord::all_within_radius(2).into_iter().collect();
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
pub struct RerollState {
    pub cost: u32,
}

#[derive(Resource, Default)]
pub struct GridHighlights {
    pub center_entity: Option<Entity>,
    pub center_pos: Option<Vec3>,
    pub write_targets: HashSet<HexCoord>,
    pub write_sources: HashSet<HexCoord>,
    pub pattern_cells: HashSet<HexCoord>,
}

pub fn can_place(is_joker: bool, target: RuneSource, grid: &RuneGrid) -> bool {
    match target {
        RuneSource::Shop(_) => true,
        RuneSource::Grid(c) => {
            if !grid.is_unlocked(c) {
                return false;
            }
            if is_joker && !is_joker_slot(c) {
                return false;
            }
            true
        }
    }
}
