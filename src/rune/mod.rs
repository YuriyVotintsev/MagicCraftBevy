mod content;
mod cost;
mod data;
mod hex;
mod shop_gen;

use bevy::prelude::*;

pub use content::{
    write_pattern_contains, write_pattern_coords, write_targets, Tier, WriteEffect,
};
pub use cost::RuneCosts;
pub use data::{
    can_place, Dragging, GridCellView, GridHighlights, IconAssets, JokerSlotView, JokerSlots,
    RerollState, Rune, RuneGrid, RuneSource, RuneView, ShopOffer, GRID_RADIUS, JOKER_SLOTS,
    SHOP_SLOTS,
};
pub use hex::HexCoord;
pub use shop_gen::{fill_shop_offer, roll_shop_offer};

use crate::game_state::GameState;
use crate::wave::WavePhase;

pub struct RunePlugin;

impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopOffer>()
            .init_resource::<RuneGrid>()
            .init_resource::<JokerSlots>()
            .init_resource::<GridHighlights>()
            .init_resource::<RerollState>()
            .add_systems(OnEnter(GameState::Playing), reset_run_content)
            .add_systems(OnEnter(WavePhase::Combat), clear_shop_offer);
    }
}

fn reset_run_content(
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    mut shop: ResMut<ShopOffer>,
    mut reroll: ResMut<RerollState>,
) {
    *grid = RuneGrid::default();
    *jokers = JokerSlots::default();
    *shop = ShopOffer::default();
    *reroll = RerollState::default();
}

fn clear_shop_offer(mut shop: ResMut<ShopOffer>) {
    shop.stubs = [None; SHOP_SLOTS];
}
