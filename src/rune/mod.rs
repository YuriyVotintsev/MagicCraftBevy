mod calc;
mod content;
mod cost;
mod data;
mod hex;
mod scene;
mod shop_gen;

use bevy::prelude::*;

pub use calc::add_grid_modifiers;
pub use content::Tier;
pub use cost::RuneCosts;
pub use data::{
    Dragging, GridHighlights, JokerSlots, RerollState, RuneGrid, RuneSource, ShopOffer,
    SHOP_SLOTS,
};
pub use scene::find_drop_target_world;
pub use shop_gen::roll_shop_offer;

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
        scene::register_systems(app);
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
