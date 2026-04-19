mod calc;
mod content;
mod data;
mod hex;
mod scene;
mod shop_gen;

use bevy::prelude::*;

pub use calc::add_grid_modifiers;
pub use content::{RuneKind, Tier};
pub use data::{
    Dragging, GridHighlights, RerollState, RuneGrid, RuneSource, ShopOffer,
    SHOP_SLOTS,
};
pub use scene::{
    find_drop_target_world, shop_grid_half_extent, BALL_RADIUS, SHOP_BALL_RING_RADIUS,
    SHOP_BALL_X,
};
pub use shop_gen::roll_shop_offer;

use crate::game_state::GameState;
use crate::wave::WavePhase;

pub struct RunePlugin;

impl Plugin for RunePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopOffer>()
            .init_resource::<RuneGrid>()
            .init_resource::<GridHighlights>()
            .init_resource::<RerollState>()
            .add_systems(OnEnter(GameState::Playing), reset_run_content)
            .add_systems(OnEnter(WavePhase::Combat), clear_shop_offer);
        scene::register_systems(app);
    }
}

fn reset_run_content(
    mut grid: ResMut<RuneGrid>,
    mut shop: ResMut<ShopOffer>,
    mut reroll: ResMut<RerollState>,
) {
    *grid = RuneGrid::default();
    *shop = ShopOffer::default();
    *reroll = RerollState::default();
}

fn clear_shop_offer(mut shop: ResMut<ShopOffer>) {
    shop.stubs = [None; SHOP_SLOTS];
}
