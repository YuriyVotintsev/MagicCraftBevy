use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::hex::HexCoord;

pub const GRID_RADIUS: i32 = 3;
pub const SHOP_SLOTS: usize = 4;
pub const JOKER_SLOTS: usize = 6;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RuneStub {
    pub id: u32,
    pub color: Color,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JokerStub {
    pub id: u32,
    pub color: Color,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RuneSource {
    Shop(usize),
    Grid(HexCoord),
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct RuneView {
    pub source: RuneSource,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct GridCellView {
    pub coord: HexCoord,
    pub center: Vec2,
    pub unlocked: bool,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct ShopSlotView {
    pub index: usize,
    pub center: Vec2,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct JokerSlotView {
    pub index: usize,
    pub center: Vec2,
}

#[derive(Resource)]
pub struct ShopOffer {
    pub runes: [Option<RuneStub>; SHOP_SLOTS],
    pub next_id: u32,
}

impl Default for ShopOffer {
    fn default() -> Self {
        Self {
            runes: [None; SHOP_SLOTS],
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
    pub slots: [Option<JokerStub>; JOKER_SLOTS],
}

#[derive(Resource, Default, Debug)]
pub enum DragState {
    #[default]
    Idle,
    Dragging {
        entity: Entity,
        from: RuneSource,
    },
}
