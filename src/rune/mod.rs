mod content;
mod cost;
mod data;
mod hex;
mod shop_gen;

use bevy::prelude::*;

pub use content::Tier;
pub use cost::RuneCosts;
pub use data::{
    can_place, Dragging, GridCellView, JokerSlotView, JokerSlots, Rune, RuneGrid, RuneSource,
    RuneView, ShopOffer, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
pub use hex::HexCoord;
pub use shop_gen::fill_shop_offer;

use crate::game_state::GameState;
use crate::wave::WavePhase;

pub struct RunePlugin;

impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopOffer>()
            .init_resource::<RuneGrid>()
            .init_resource::<JokerSlots>()
            .add_systems(OnEnter(GameState::Playing), reset_run_content)
            .add_systems(OnEnter(WavePhase::Combat), clear_shop_offer);
    }
}

fn reset_run_content(
    mut grid: ResMut<RuneGrid>,
    mut jokers: ResMut<JokerSlots>,
    mut shop: ResMut<ShopOffer>,
) {
    *grid = RuneGrid::default();
    *jokers = JokerSlots::default();
    *shop = ShopOffer::default();
}

fn clear_shop_offer(mut shop: ResMut<ShopOffer>) {
    shop.stubs = [None; SHOP_SLOTS];
}
