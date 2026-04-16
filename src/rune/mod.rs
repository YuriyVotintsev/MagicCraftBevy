mod data;
mod hex;
mod shop_gen;

use bevy::prelude::*;

pub use data::{
    can_place, Dragging, GridCellView, JokerSlotView, JokerSlots, RuneGrid, RuneSource, RuneStub,
    RuneView, ShopOffer, ShopSlotView, StubKind, GRID_RADIUS, JOKER_SLOTS, SHOP_SLOTS,
};
pub use hex::HexCoord;
pub use shop_gen::fill_shop_offer;

pub struct RunePlugin;

impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopOffer>()
            .init_resource::<RuneGrid>()
            .init_resource::<JokerSlots>();
    }
}
